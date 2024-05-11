use mapp::prelude::*;
use msysev::*;
use serde::Deserialize;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::warn;

use mtool_core::{AppStage, ConfigStore};

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(Observer::new);

        app.schedule().add_once_task(AppStage::Run, run_event_loop);
        Ok(())
    }
}

fn default_channel_size() -> usize {
    1024
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    #[serde(default = "default_channel_size")]
    channel_size: usize,
}

pub struct Observer {
    tx: Sender<Event>,
    exit_signal: ExitSignal,
}

impl Observer {
    async fn new(injector: Injector, cs: Res<ConfigStore>) -> Result<Res<Self>, anyhow::Error> {
        let config = cs.get::<Config>("system.event").await?;

        let (tx, _) = broadcast::channel(config.channel_size);

        let event_loop = EventLoop::new()?;
        let exit_signal = event_loop.exit_signal();

        injector.insert(Take::new(event_loop));

        Ok(Res::new(Self { tx, exit_signal }))
    }

    pub fn subscribe(&self) -> Receiver<Event> {
        self.tx.subscribe()
    }

    #[allow(dead_code)]
    pub fn receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }

    pub fn close(&self) -> Result<(), anyhow::Error> {
        self.exit_signal.exit();
        Ok(())
    }
}

async fn run_event_loop(
    event_loop: TakeOpt<EventLoop>,
    observer: Res<Observer>,
) -> Result<(), anyhow::Error> {
    let tx = observer.tx.clone();
    if let Some(event_loop) = event_loop.unwrap() {
        event_loop
            .take()?
            .run(move |ev| -> ControlFlow {
                if let Err(e) = tx.send(ev) {
                    warn!(
                        "send system event error: {}, receiver count {}",
                        e,
                        tx.receiver_count()
                    );
                }
                ControlFlow::Continue(())
            })
            .await?
    }
    Ok(())
}
