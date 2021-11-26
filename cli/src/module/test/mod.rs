use crate::{
    app::App,
    core::{command::Command, keybind::define_globale_key},
};

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
        log::info!("test");
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

pub async fn define_key(app: &mut App) -> anyhow::Result<()> {
    define_globale_key(app, "C-j t", "test").await?;
    Ok(())
}

pub async fn module_load(app: &mut App) -> anyhow::Result<()> {
    add_command(app)?;
    define_key(app).await?;
    Ok(())
}
