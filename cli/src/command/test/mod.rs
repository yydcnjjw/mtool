use std::thread;

use crate::app::App;
use sysev::{self, event::Event};
use tokio::sync::broadcast;

use super::Command;
use async_trait::async_trait;

struct Cmd {}

impl Cmd {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Cmd {
    async fn exec(&mut self, _: Vec<String>) -> anyhow::Result<()> {
        let (tx, mut rx) = broadcast::channel(20);

        thread::spawn(|| {
            sysev::run_loop(tx).unwrap();
        });

        loop {
            let ev = rx.recv().await;
            if let Err(e) = ev {
                println!("{}", e);
                break;
            }

            let ev = ev.unwrap();

            
            match ev {
                Event::Key(_) => todo!(),
            }
        }
        Ok(())
    }
}

pub fn add_command(app: &mut App) -> anyhow::Result<()> {
    {
        let cmd = Box::new(Cmd::new());
        app.cmder.insert("test".into(), cmd);
    }

    Ok(())
}
