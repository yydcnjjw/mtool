use async_trait::async_trait;
use mapp::prelude::*;
use mkeybinding::KeySequence;
use mtool_core::{
    config::{not_startup_mode, StartupMode},
    AppStage,
};

use super::{GlobalHotKeyStage, Keybinding, SetGlobalHotKey};

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(GlobalHotKeyMgr::construct);

        app.schedule().insert_stage_vec_with_cond(
            AppStage::Init,
            vec![GlobalHotKeyStage::Register, GlobalHotKeyStage::Setup],
            not_startup_mode(StartupMode::Cli),
        );
        app.schedule().insert_stage_with_cond(
            AppStage::Run,
            GlobalHotKeyStage::UnRegister,
            not_startup_mode(StartupMode::Cli),
        );
        app.schedule()
            .add_once_task(GlobalHotKeyStage::Setup, GlobalHotKeyMgr::setup);
        Ok(())
    }
}

pub struct GlobalHotKeyMgr {
    ks_lst: Vec<KeySequence>,
}

impl GlobalHotKeyMgr {
    const GLOBAL_KEYMAP: &'static str = "global";

    async fn construct(injector: Injector) -> Result<Res<Keybinding>, anyhow::Error> {

        // Keybinding::new(Res::new(Self { ks_lst: Vec::new() }), rx)
        
        Ok()
    }

    async fn setup(self: Res<GlobalHotKeyMgr>) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

#[async_trait]
impl SetGlobalHotKey for GlobalHotKeyMgr {
    async fn register(&self, ks: &KeySequence) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn unregister(&self, ks: &KeySequence) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
