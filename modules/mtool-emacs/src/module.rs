use emacs::Env;
use mapp::{
    anyhow,
    prelude::*,
    sync::Mutex,
    tokio::{self, sync::oneshot},
};
use mtool_core::ConfigStore;

use crate::{context::EmacsContext, user_idle};

type EmacsContextCtor = Box<dyn FnOnce(&Env) -> Result<EmacsContext, anyhow::Error> + Send + Sync>;

pub(crate) struct EmacsModule {
    config_dir: String,
    tx: Mutex<Option<oneshot::Sender<EmacsContextCtor>>>,
}

impl EmacsModule {
    pub fn new(env: &Env) -> Result<(Self, oneshot::Receiver<EmacsContextCtor>), anyhow::Error> {
        let config_dir = env
            .call("eval", [env.intern("my/mtool-config-dir")?])?
            .into_rust()?;
        let (tx, rx) = oneshot::channel();
        Ok((
            Self {
                config_dir,
                tx: Mutex::new(Some(tx)),
            },
            rx,
        ))
    }
}

#[async_trait]
impl AppModule for EmacsModule {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        {
            let injector = ctx.injector().clone();
            let rt = tokio::runtime::Handle::current();
            _ = self.tx.lock().take().unwrap().send(Box::new(
                move |_env| -> Result<EmacsContext, anyhow::Error> {
                    Ok(EmacsContext { injector, rt })
                },
            ));
        }

        ctx.injector()
            .insert(Res::new(ConfigStore::new(&self.config_dir).await?));
        Ok(())
    }
}

#[cfg(feature = "graphic")]
pub fn module() -> ModuleGroup {
    let mut group = ModuleGroup::new("emacs");

    group.add_module(user_idle::Module);

    group
}
