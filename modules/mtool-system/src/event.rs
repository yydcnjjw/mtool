use async_trait::async_trait;
use mapp::{provider::Res, AppContext, AppModule};
use serde::Deserialize;
use std::thread::{self, JoinHandle};
use tokio::sync::{
    broadcast::{self, Receiver, Sender},
    Mutex,
};

use mtool_core::{AppStage, ConfigStore};

pub use msysev::Event;

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(Observer::new);

        app.schedule().add_once_task(AppStage::Exit, wait_for_exit);
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

type Worker = JoinHandle<Result<(), anyhow::Error>>;

pub struct Observer {
    tx: Sender<Event>,
    worker: Mutex<Option<Worker>>,
}

impl Observer {
    async fn new(cs: Res<ConfigStore>) -> Result<Res<Self>, anyhow::Error> {
        let config = cs.get::<Config>("system.event").await?;

        let (tx, worker) = run_loop(config.channel_size);

        Ok(Res::new(Self {
            tx,
            worker: Mutex::new(Some(worker)),
        }))
    }

    pub fn subscribe(&self) -> Receiver<Event> {
        self.tx.subscribe()
    }

    #[allow(dead_code)]
    pub fn receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }

    #[allow(dead_code)]
    pub fn close(&self) -> Result<(), anyhow::Error> {
        msysev::quit()
    }
}

fn run_loop(size: usize) -> (Sender<Event>, Worker) {
    let (tx, _) = broadcast::channel(size);
    let tx_ = tx.clone();

    let worker = thread::spawn(move || {
        msysev::run_loop(move |e| {
            if let Err(e) = tx.send(e) {
                log::warn!(
                    "send system event error: {}, receiver count {}",
                    e,
                    tx.receiver_count()
                );
            }
            Ok(())
        })
    });

    (tx_, worker)
}

async fn wait_for_exit(ob: Option<Res<Observer>>) -> Result<(), anyhow::Error> {
    if let Some(ob) = ob {
        ob.worker
            .lock()
            .await
            .take()
            .unwrap()
            .join()
            .map_err(|_| anyhow::anyhow!("waiting for system event loop"))??;
    }
    Ok(())
}
