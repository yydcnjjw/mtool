use mapp::{
    anyhow,
    prelude::*,
    serde::Deserialize,
    sync::Mutex,
    tokio::{
        self,
        sync::broadcast::{self, Receiver, Sender},
    },
    tracing::warn,
};
pub use msysev::prelude::*;

use msysev::{ControlFlow, EventLoop, ExitSignal};
use mtool_core::ConfigStore;

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(Observer::construcct);
        Ok(())
    }
}

fn default_channel_size() -> usize {
    1024
}

#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "mapp::serde")]
struct Config {
    #[serde(default = "default_channel_size")]
    channel_size: usize,
}

pub struct Observer {
    tx: Sender<Event>,
    exit_signal: Mutex<Option<ExitSignal>>,
}

impl Observer {
    async fn construcct(cs: Res<ConfigStore>) -> Result<Res<Self>, anyhow::Error> {
        let config = cs.get::<Config>("system.event")?;

        let (tx, _) = broadcast::channel(config.channel_size);

        Ok(Res::new(Self {
            tx,
            exit_signal: Mutex::new(None),
        }))
    }

    pub fn subscribe(&self) -> Receiver<Event> {
        let rx = self.tx.subscribe();
        if self.exit_signal.lock().is_none() {
            self.run_event_loop();
        }
        rx
    }

    pub fn run_event_loop(&self) {
        match || -> Result<ExitSignal, anyhow::Error> {
            let tx = self.tx.clone();
            let event_loop = EventLoop::new()?;
            let exit_signal = event_loop.exit_signal();

            tokio::spawn(async move {
                if let Err(e) = event_loop
                    .run(move |ev| -> ControlFlow {
                        let _ = tx.send(ev);
                        ControlFlow::Continue(())
                    })
                    .await
                {
                    warn!("{:?}", e);
                }
            });
            Ok(exit_signal)
        }() {
            Err(e) => {
                warn!("{:?}", e);
            }
            Ok(exit_signal) => *self.exit_signal.lock() = Some(exit_signal),
        }
    }

    #[allow(dead_code)]
    pub fn receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }

    pub fn close(&self) -> Result<(), anyhow::Error> {
        if let Some(signal) = self.exit_signal.lock().take() {
            signal.exit()
        }
        Ok(())
    }
}
