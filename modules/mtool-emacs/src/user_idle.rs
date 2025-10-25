use emacs::{defun, Env};
use mapp::{
    anyhow,
    futures::future,
    prelude::*,
    serde::{Deserialize, Serialize},
    tokio,
    tokio_stream::StreamExt,
    tracing::warn,
    CreateOnceTaskDescriptor,
};
use mtool_core::{AppStage, ConfigStore};
use mtool_p2p::{self as p2p, gossipsub::IdentTopic, SubjectMessage};
use mtool_system::{SystemEvent, SystemEventSource};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        LazyLock,
    },
    time::Duration,
};

use crate::context::EmacsContext;

#[defun(mod_in_name = false)]
fn user_idle(_env: &Env, _ctx: &EmacsContext) -> Result<u64, emacs::Error> {
    Ok(USER_IDLE.idle_time.load(Ordering::Relaxed))
}

pub(crate) struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule()
            .add_once_task(AppStage::Run, subscribe_user_idle)
            .add_once_task(
                AppStage::Run,
                broadcast_user_idle.cond(|cs: Res<ConfigStore>| {
                    future::ok(cs.get_optional("emacs.user_idle.broadcast").unwrap_or(true))
                }),
            );
        Ok(())
    }
}

static IDLE_TIME_TOPIC: LazyLock<IdentTopic> = LazyLock::new(|| IdentTopic::new("IDLE_TIME"));
static USER_IDLE: LazyLock<UserIdle> = LazyLock::new(|| UserIdle::new());

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "mapp::serde")]
struct UserIdle {
    idle_time: AtomicU64,
}

impl Clone for UserIdle {
    fn clone(&self) -> Self {
        Self {
            idle_time: AtomicU64::new(self.idle_time.load(Ordering::Relaxed)),
        }
    }
}

impl UserIdle {
    fn new() -> Self {
        Self {
            idle_time: AtomicU64::new(0),
        }
    }

    fn add(&self, time: u64) -> Self {
        self.idle_time.fetch_add(time, Ordering::Relaxed);
        self.to_owned()
    }

    fn is_active(&self) -> bool {
        self.idle_time.load(Ordering::Relaxed) == 0
    }

    fn active(&self) -> Self {
        self.idle_time.store(0, Ordering::Relaxed);
        self.to_owned()
    }

    fn update(&self, rhs: &Self) {
        self.idle_time
            .store(rhs.idle_time.load(Ordering::Relaxed), Ordering::Relaxed);
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "mapp::serde")]
enum Message {
    Update(UserIdle),
}

async fn subscribe_user_idle(peer: Res<p2p::Peer>) -> Result<(), anyhow::Error> {
    let subject = peer.subscribe::<Message>(&IDLE_TIME_TOPIC).await?;

    let mut stream = subject.stream();
    tokio::spawn(async move {
        while let Some(item) = stream.next().await {
            match item {
                Ok(SubjectMessage { data, .. }) => match data {
                    Message::Update(update) => {
                        USER_IDLE.update(&update);
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

async fn broadcast_user_idle(
    peer: Res<p2p::Peer>,
    source: Res<SystemEventSource>,
) -> Result<(), anyhow::Error> {
    tokio::spawn(async move {
        let mut rx = source.subscribe();

        let check_idle_interval = 5; // s
        let mut timer = tokio::time::interval(Duration::from_secs(check_idle_interval));

        loop {
            if let Err(e) = tokio::select! {
                Ok(SystemEvent::Keyboard(keyboard)) = rx.recv() => {
                    if !USER_IDLE.is_active() {
                        peer.publish(&IDLE_TIME_TOPIC, &Message::Update(USER_IDLE.active())).await
                    } else {
                        Ok(())
                    }
                }
                _ = timer.tick() => {
                    peer.publish(&IDLE_TIME_TOPIC, &Message::Update(USER_IDLE.add(check_idle_interval))).await
                }
            } {
                warn!("{e:?}");
            }
        }
    });

    Ok(())
}
