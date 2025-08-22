use mapp::{
    anyhow::{self, Context},
    define_label,
    futures::{channel::oneshot, TryFutureExt},
    prelude::*,
};
use mtool_core::{AppStage, CmdlineStage, ConfigStore};
use tonic::transport::Server;

use crate::{Config, Router};

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("rpc_group");

    group.add_module(Module);

    group
}

struct Module;

define_label!(
    pub enum RpcStage {
        Setup,
    }
);

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().insert(Res::new(Router::new()));
        ctx.schedule()
            .insert_stage_vec(CmdlineStage::AfterParse, vec![RpcStage::Setup])
            .add_once_task(AppStage::Run, run);
        Ok(())
    }
}

async fn run(
    router: Take<Res<Router>>,
    cs: Res<ConfigStore>,
    exit_signal: Res<ExitSignal>,
) -> Result<(), anyhow::Error> {
    let cfg = cs.get_optional::<Config>("rpc").unwrap_or_default();

    let routes = router.take()?.routes();
    let listen = cfg.listen.parse()?;

    let (tx, rx) = oneshot::channel();

    exit_signal.add_handler(move || {
        _ = tx.send(());
    });

    Server::builder()
        .add_routes(routes)
        .serve_with_shutdown(listen, rx.unwrap_or_else(|_| ()))
        .await
        .context("Failed to serve RPC server")
}
