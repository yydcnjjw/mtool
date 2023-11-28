mod db_conn;
mod migration;

use db_conn::create_db_conn;
pub use migration::*;

use async_trait::async_trait;
use mapp::prelude::*;
use mtool_core::{AppStage, CmdlineStage};

#[derive(Default)]
struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(create_db_conn);

        app.schedule()
            .insert_stage(AppStage::Startup, DBMigrationStage::Register)
            .insert_stage(CmdlineStage::AfterInit, DBMigrationStage::Migrate)
            .add_once_task(DBMigrationStage::Migrate, migrate);
        Ok(())
    }
}

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-storage");
    group.add_module(Module);
    group
}
