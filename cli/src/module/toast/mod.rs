struct Toast {}

#[async_trait]
impl Command for Toast {
    async fn exec(&mut self, args: Vec<String>) -> anyhow::Result<command::Output> {
        
    }
}

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    let sender = &app.evbus.sender();
    AddCommand::post(sender, "notify".into(), Toast {}).await?;
    Ok(())
}
