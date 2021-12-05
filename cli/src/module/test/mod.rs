use crate::{
    app::App,
    core::{
        command::{AddCommand, Command},
        keybind::DefineKeyBinding,
    },
};

use async_trait::async_trait;

struct TestCmd {}
#[async_trait]
impl Command for TestCmd {
    async fn exec(&mut self, _args: Vec<String>) -> anyhow::Result<()> {
        println!("test");
        Ok(())
    }
}

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    let sender = &app.evbus.sender();
    AddCommand::post(sender, "test".into(), TestCmd {}).await?;
    DefineKeyBinding::post(sender, "C-m t", "test").await??;
    Ok(())
}
