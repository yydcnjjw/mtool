use crate::core::{
    command::ExecCommand,
    evbus::{Event, Receiver, Sender},
};

#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub kbd: String,
    pub cmd_name: String,
}

pub struct KeyBindinger {}

impl KeyBindinger {
    pub async fn run_loop(sender: Sender, mut rx: Receiver) {
        while let Ok(e) = rx.recv().await {
            if let Some(e) = e.downcast_ref::<Event<KeyBinding>>() {
                let tx = sender.clone();
                let cmd_name = e.cmd_name.clone();
                tokio::spawn(async move {
                    if let Err(e) = ExecCommand::post(&tx, cmd_name, Vec::new()).await {
                        log::error!("{}", e);
                    }
                });
            }
        }
    }
}
