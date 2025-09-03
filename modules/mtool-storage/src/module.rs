use mapp::{anyhow, futures::future, prelude::*, CreateOnceTaskDescriptor};

use mtool_core::{AppStage, CmdlineStage};

use crate::{crdt::CrdtStore, create_kvstore, lww::LwwStore, DBMigrationStage};

struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector()
            .construct_once(create_kvstore)
            .construct_once(CrdtStore::construct)
            .construct_once(LwwStore::construct);

        app.schedule()
            .insert_stage(AppStage::Startup, DBMigrationStage::Register)
            .insert_stage(CmdlineStage::AfterParse, DBMigrationStage::Migrate);
        Ok(())
    }
}

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-storage");
    group.add_module(Module);
    group
}
