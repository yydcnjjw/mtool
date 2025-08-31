use mapp::{anyhow, prelude::*};

use crate::{p2p::RemoteSystemEventSource, SystemEventSource};

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(SystemEventSource::construct);
        app.injector()
            .construct_once(RemoteSystemEventSource::construct);
        Ok(())
    }
}

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("mtool-system");

    group.add_module(Module);

    group
}
