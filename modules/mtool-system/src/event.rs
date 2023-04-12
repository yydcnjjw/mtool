use async_trait::async_trait;
use mapp::{
    provider::{Injector, Res, Take, TakeOpt},
    AppContext, AppModule,
};
use serde::Deserialize;
use tokio::{
    sync::broadcast::{self, Receiver, Sender},
    task::JoinHandle,
};
use tracing::warn;

use mtool_core::{AppStage, ConfigStore};

pub use msysev::Event;

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(Observer::new);

        app.schedule().add_once_task(AppStage::Run, wait_for_exit);
        Ok(())
    }
}

fn default_channel_size() -> usize {
    10
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    #[serde(default = "default_channel_size")]
    channel_size: usize,
}

pub struct Observer {
    tx: Sender<Event>,
}

struct Worker(JoinHandle<Result<(), anyhow::Error>>);

impl Observer {
    async fn new(injector: Injector, cs: Res<ConfigStore>) -> Result<Res<Self>, anyhow::Error> {
        let config = cs.get::<Config>("system.event").await?;

        let (tx, worker) = run_loop(config.channel_size);

        injector.insert(Take::new(worker));

        Ok(Res::new(Self { tx }))
    }

    pub fn subscribe(&self) -> Receiver<Event> {
        self.tx.subscribe()
    }

    #[allow(dead_code)]
    pub fn receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }

    pub fn close(&self) -> Result<(), anyhow::Error> {
        msysev::quit()
    }
}

fn run_loop(size: usize) -> (Sender<Event>, Worker) {
    let (tx, _) = broadcast::channel(size);
    let tx_ = tx.clone();

    let worker = tokio::task::spawn_blocking(move || {
        msysev::run_loop(move |e| {
            if let Err(e) = tx.send(e) {
                warn!(
                    "send system event error: {}, receiver count {}",
                    e,
                    tx.receiver_count()
                );
            }
            Ok(())
        })
    });

    (tx_, Worker(worker))
}

async fn wait_for_exit(worker: TakeOpt<Worker>) -> Result<(), anyhow::Error> {
    if let Some(worker) = worker.unwrap() {
        worker.take()?.0.await??;
    }
    Ok(())
}
