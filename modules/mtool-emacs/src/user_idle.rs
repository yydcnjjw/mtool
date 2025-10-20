use emacs::{defun, Env};
use mapp::{
    anyhow,
    once_cell::sync::OnceCell,
    prelude::*,
    serde::{Deserialize, Serialize},
    sync::RwLock,
    tokio,
    tokio_stream::StreamExt,
    tracing::warn,
};
use mtool_core::AppStage;
use mtool_p2p::{self as p2p, gossipsub::IdentTopic, SubjectMessage};
use std::{sync::LazyLock, time::Duration};

use crate::context::EmacsContext;

#[defun(mod_in_name = false)]
fn user_idle(_env: &Env, _ctx: &EmacsContext) -> Result<u64, emacs::Error> {
    Ok(USER_IDLE
        .get_or_init(|| RwLock::new(UserIdle::new()))
        .read()
        .idle_time)
}

pub(crate) struct EmacsModule;

#[async_trait]
impl AppModule for EmacsModule {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule()
            .add_once_task(AppStage::Run, subscribe_user_idle);
        Ok(())
    }
}

static IDLE_TIME_TOPIC: LazyLock<IdentTopic> = LazyLock::new(|| IdentTopic::new("IDLE_TIME"));

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "mapp::serde")]
struct UserIdle {
    idle_time: u64,
}

impl UserIdle {
    fn new() -> Self {
        Self { idle_time: 0 }
    }

    fn add(&mut self, time: u64) -> UserIdle {
        self.idle_time += time;
        UserIdle {
            idle_time: self.idle_time,
        }
    }

    fn is_active(&self) -> bool {
        self.idle_time == 0
    }

    fn active(&mut self) -> UserIdle {
        self.idle_time = 0;
        UserIdle { idle_time: 0 }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "mapp::serde")]
enum Message {
    Update(UserIdle),
}

static USER_IDLE: OnceCell<RwLock<UserIdle>> = OnceCell::new();

async fn subscribe_user_idle(peer: Res<p2p::Peer>) -> Result<(), anyhow::Error> {
    let subject = peer.subscribe::<Message>(&IDLE_TIME_TOPIC).await?;

    let mut stream = subject.stream();
    tokio::spawn(async move {
        while let Some(item) = stream.next().await {
            match item {
                Ok(SubjectMessage { data, .. }) => match data {
                    Message::Update(update) => {
                        if let Some(user_idle) = USER_IDLE.get() {
                            *user_idle.write() = update;
                        }
                    }
                },
                Err(e) => {
                    warn!("{e:?}");
                    break;
                }
            }
        }
    });

    Ok(())
}

#[cfg(feature = "graphic")]
pub(crate) struct Module;

#[cfg(feature = "graphic")]
#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule()
            .add_once_task(AppStage::Run, broadcast_user_idle);

        use mtool_system::{SystemEvent, SystemEventSource};

        async fn broadcast_user_idle(
            peer: Res<p2p::Peer>,
            source: Res<SystemEventSource>,
        ) -> Result<(), anyhow::Error> {
            tokio::spawn(async move {
                let mut rx = source.subscribe();
                let mut user_idle = UserIdle::new();

                let check_idle_interval = 5; // s
                let mut timer = tokio::time::interval(Duration::from_secs(check_idle_interval));

                loop {
                    if let Err(e) = tokio::select! {
                        Ok(SystemEvent::Keyboard(_)) = rx.recv() => {
                            if !user_idle.is_active() {
                                peer.publish(&IDLE_TIME_TOPIC, &Message::Update(user_idle.active())).await
                            } else {
                                Ok(())
                            }
                        }
                        _ = timer.tick() => {
                            peer.publish(&IDLE_TIME_TOPIC, &Message::Update(user_idle.add(check_idle_interval))).await
                        }
                    } {
                        warn!("{e:?}");
                    }
                }
            });

            Ok(())
        }

        Ok(())
    }
}
