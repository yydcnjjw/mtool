use std::sync::Arc;

use mapp::{
    anyhow, define_label,
    prelude::*,
    tokio::{self, sync::oneshot, task::JoinHandle},
};
use mtool_core::{AppStage, ConfigStore};

use crate::{network, Config, Peer};

pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("p2p");

    group.add_module(Module);

    group
}

struct Module;

define_label!(
    pub enum P2pStage {
        Setup,
        Init,
    }
);

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        let peer_tx = ctx.injector().construct_oneshot();

        ctx.schedule()
            .insert_stage_vec(AppStage::BeforeInit, vec![P2pStage::Setup, P2pStage::Init])
            .add_once_task(P2pStage::Init, move |cs, injector, exit_signal| {
                init(cs, injector, exit_signal, peer_tx)
            })
            .add_once_task(AppStage::Run, run);
        Ok(())
    }
}

struct EventLoopWorker(JoinHandle<()>);

async fn init(
    cs: Res<ConfigStore>,
    injector: Injector,
    exit_signal: Res<ExitSignal>,
    peer_tx: oneshot::Sender<Res<Peer>>,
) -> Result<(), anyhow::Error> {
    let cfg = cs.get_optional::<Config>("p2p").unwrap_or_default();

    let (peer, event_loop) = network::new(cfg)?;

    let worker = tokio::spawn(event_loop.run());
    injector.insert(Arc::new(EventLoopWorker(worker)));

    let peer = Res::new(peer);
    _ = peer_tx.send(peer.clone());

    exit_signal.add_handler(move || {
        peer.shutdown();
    });

    Ok(())
}

async fn run(event_loop: Take<Arc<EventLoopWorker>>) -> Result<(), anyhow::Error> {
    Ok(event_loop.take()?.0.await?)
}
