use mapp::{prelude::*, tokio::sync::mpsc, CreateOnceTaskDescriptor};
use mkeybinding::{KeySequence, ToKeySequence};
use mtool_core::{
    config::{not_startup_mode, StartupMode},
    AppStage,
};

use super::{GlobalHotKeyEvent, Keybinding, SetupGlobalHotKey};

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(GlobalHotKeyMgr::construct);

        app.schedule().add_once_task(
            AppStage::Run,
            GlobalHotKeyMgr::run.cond(not_startup_mode(StartupMode::Cli)),
        );
        Ok(())
    }
}

#[derive(Clone)]
pub struct GlobalHotKeyMgr {
    sender: mpsc::UnboundedSender<GlobalHotKeyEvent>,
}

#[zbus::interface(name = "org.yydcnjjw.mtool.GlobalHotKey1")]
impl GlobalHotKeyMgr {
    async fn emit(&self, ks: &str) -> zbus::fdo::Result<()> {
        let _ = self.sender.send(GlobalHotKeyEvent(
            ks.to_key_sequence()
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?,
        ));
        Ok(())
    }
}

impl GlobalHotKeyMgr {
    async fn construct(injector: Injector) -> Result<Res<Keybinding>, anyhow::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let hotkey_mgr = Self::new(tx);
        injector.insert(Res::new(hotkey_mgr.clone()));

        let keybinding = Res::new(Keybinding::new(Res::new(hotkey_mgr), rx));
        tokio::spawn(keybinding.clone().handle_event_loop(injector));

        Ok(keybinding)
    }

    fn new(sender: mpsc::UnboundedSender<GlobalHotKeyEvent>) -> Self {
        Self { sender }
    }

    async fn run(
        this: TakeOpt<Res<GlobalHotKeyMgr>>,
        injector: Injector,
    ) -> Result<(), anyhow::Error> {
        if let Some(this) = this.unwrap() {
            let conn = zbus::connection::Builder::session()?
                .name("org.yydcnjjw.mtool.GlobalHotKey")?
                .serve_at(
                    "/org/yydcnjjw/mtool/GlobalHotKey",
                    Res::try_unwrap(this.take()?)?,
                )?
                .build()
                .await?;
            injector.insert(conn);
        }
        Ok(())
    }
}

#[async_trait]
impl SetupGlobalHotKey for GlobalHotKeyMgr {
    async fn register(&self, _: &KeySequence) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn unregister(&self, _: &KeySequence) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
