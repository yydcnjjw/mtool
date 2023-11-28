mod m20231128_120000_create_table;

use async_trait::async_trait;
use mapp::prelude::*;
use mtool_storage::{add_migration, DBMigrationStage};

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule()
            .add_once_task(DBMigrationStage::Register, register);
        Ok(())
    }
}

async fn register() -> Result<(), anyhow::Error> {
    add_migration(m20231128_120000_create_table::Migration);
    Ok(())
}
