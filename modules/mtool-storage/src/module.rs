use crate::{crdt::CrdtService, create_kvstore, rpc::RpcService, DBMigrationStage};
use mapp::{anyhow, prelude::*};

use mtool_core::{AppStage, CmdlineStage};
use mtool_rpc::RpcStage;

struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector()
            .construct_once(create_kvstore)
            .construct_once(CrdtService::construct);

        app.schedule()
            .insert_stage(AppStage::Startup, DBMigrationStage::Register)
            .insert_stage(CmdlineStage::AfterParse, DBMigrationStage::Migrate)
            .add_once_task(RpcStage::Setup, RpcService::setup);
        // .add_once_task(DBMigrationStage::Migrate, migrate);
        Ok(())
    }
}

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-storage");
    group.add_module(Module);
    group
}
