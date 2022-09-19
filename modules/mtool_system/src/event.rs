use async_trait::async_trait;
use clap::{arg, value_parser, ArgMatches};
use mapp::{AppContext, AppModule, CreateTaskDescriptor, Res};
use std::thread::{self, JoinHandle};
use tokio::sync::{
    broadcast::{self, Receiver, Sender},
    Mutex,
};

use mtool_core::{config::is_daemon, Cmdline, ExitStage, StartupStage};

pub use msysev::Event;

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct(Observer::new).await;

        app.schedule()
            .add_task(StartupStage::Startup, setup_cmdline)
            .await
            .add_task(ExitStage::Exit, exit.cond(is_daemon))
            .await;
        Ok(())
    }
}

type Worker = JoinHandle<Result<(), anyhow::Error>>;

pub struct Observer {
    tx: Sender<Event>,
    worker: Mutex<Option<Worker>>,
}

impl Observer {
    async fn new(args: Res<ArgMatches>) -> Result<Res<Self>, anyhow::Error> {
        let (tx, worker) = run_loop(*args.get_one::<usize>("system-event-channel-size").unwrap());

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

async fn setup_cmdline(cmdline: Res<Cmdline>) -> Result<(), anyhow::Error> {
    cmdline
        .setup(|cmdline| {
            Ok(cmdline.arg(
                arg!(--"system-event-channel-size" [usize] "system event channel size")
                    .value_parser(value_parser!(usize))
                    .default_value("10"),
            ))
        })
        .await?;

    Ok(())
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

async fn exit(ob: Res<Observer>) -> Result<(), anyhow::Error> {
    ob.worker
        .lock()
        .await
        .take()
        .unwrap()
        .join()
        .map_err(|_| anyhow::anyhow!("waiting for system event loop"))?
}
