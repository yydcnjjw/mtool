use tracing::{debug, error};

use crate::{
    module::{LocalModule, LocalModuleGroup, Module, ModuleGroup},
    provider::{Injector, LocalInjector, Res},
    tracing::Tracing,
    LocalSchedule, Schedule,
};

pub struct AppBuilder {
    modules: ModuleGroup,
    tracing: Option<Tracing>,
}

impl AppBuilder {
    pub fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            modules: ModuleGroup::new("app_group"),
            tracing: Some(Tracing::new()?),
        })
    }

    fn empty() -> Self {
        Self {
            modules: Default::default(),
            tracing: None,
        }
    }

    pub fn add_module<Mod>(&mut self, module: Mod) -> &mut Self
    where
        Mod: Module + 'static,
    {
        self.modules.add_module(module);
        self
    }

    pub fn build(&mut self) -> AppRunner {
        let builder = std::mem::replace(self, AppBuilder::empty());

        let mut app_runner = AppRunner::new();

        app_runner.runner = Some(Box::new(move || -> Result<(), anyhow::Error> {
            let mut ctx = AppContext::new();

            ctx.injector().insert(Res::new(builder.tracing.unwrap()));

            let modules = builder.modules;

            modules.early_init(&mut ctx)?;

            #[cfg(target_family = "wasm")]
            let mut rt = tokio::runtime::Builder::new_current_thread();

            #[cfg(not(target_family = "wasm"))]
            let mut rt = tokio::runtime::Builder::new_multi_thread();

            let run = || async move {
                modules.init(&mut ctx).await?;

                let sche = ctx.schedule;

                let mut app = App::new();
                app.injector = ctx.injector;

                sche.run(&app).await?;

                debug!("App running!");
                Ok::<(), anyhow::Error>(())
            };

            rt.enable_all().build()?.block_on(async move {
                if let Err(e) = run().await {
                    error!("{:?}", e);
                    eprintln!("{:?}", e);
                    std::process::exit(-1);
                }
            });
            Ok(())
        }));

        app_runner
    }
}

pub struct AppContext {
    injector: Injector,
    schedule: Schedule,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            injector: Injector::new(),
            schedule: Schedule::new(),
        }
    }

    pub fn schedule(&mut self) -> &Schedule {
        return &self.schedule;
    }

    pub fn injector(&self) -> &Injector {
        return &self.injector;
    }
}

#[derive(Clone)]
pub struct App {
    injector: Injector,
}

impl App {
    pub fn new() -> Self {
        App::empty()
    }

    pub fn empty() -> Self {
        Self {
            injector: Injector::new(),
        }
    }

    pub fn injector(&self) -> &Injector {
        return &self.injector;
    }
}

pub struct LocalAppBuilder {
    modules: LocalModuleGroup,
    tracing: Option<Tracing>,
}

impl LocalAppBuilder {
    pub fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            modules: LocalModuleGroup::new("local_app_group"),
            tracing: Some(Tracing::new()?),
        })
    }

    fn empty() -> Self {
        Self {
            modules: Default::default(),
            tracing: None,
        }
    }

    pub fn add_module<Mod>(&mut self, module: Mod) -> &mut Self
    where
        Mod: LocalModule + 'static,
    {
        self.modules.add_module(module);
        self
    }

    pub fn build(&mut self) -> AppRunner {
        let builder = std::mem::replace(self, LocalAppBuilder::empty());

        let mut app_runner = AppRunner::new();

        app_runner.runner = Some(Box::new(move || -> Result<(), anyhow::Error> {
            let mut ctx = LocalAppContext::new();

            ctx.injector().insert(Res::new(builder.tracing.unwrap()));

            let modules = builder.modules;

            modules.local_early_init(&mut ctx)?;

            let mut rt = tokio::runtime::Builder::new_current_thread();

            let run = || async move {
                modules.local_init(&mut ctx).await?;

                let sche = ctx.schedule;

                let mut app = LocalApp::new();
                app.injector = ctx.injector;

                sche.run(&app).await?;

                debug!("App running!");
                Ok::<(), anyhow::Error>(())
            };

            rt.enable_all().build()?.block_on(async move {
                if let Err(e) = run().await {
                    error!("{:?}", e);
                    eprintln!("{:?}", e);
                    std::process::exit(-1);
                }
            });
            Ok(())
        }));

        app_runner
    }
}

pub struct LocalAppContext {
    injector: LocalInjector,
    schedule: LocalSchedule,
}

impl LocalAppContext {
    pub fn new() -> Self {
        Self {
            injector: LocalInjector::new(),
            schedule: LocalSchedule::new(),
        }
    }

    pub fn schedule(&mut self) -> &LocalSchedule {
        return &self.schedule;
    }

    pub fn injector(&self) -> &LocalInjector {
        return &self.injector;
    }
}

pub struct AppRunner {
    runner: Option<Box<dyn FnOnce() -> Result<(), anyhow::Error>>>,
}

impl AppRunner {
    pub fn new() -> Self {
        Self { runner: None }
    }

    pub fn run(self) {
        if let Err(e) = (self.runner.unwrap())() {
            error!("{:?}", e);
        }
    }
}

#[derive(Clone)]
pub struct LocalApp {
    injector: LocalInjector,
}

impl LocalApp {
    pub fn new() -> Self {
        Self::empty()
    }

    pub fn empty() -> Self {
        Self {
            injector: LocalInjector::new(),
        }
    }

    pub fn injector(&self) -> &LocalInjector {
        return &self.injector;
    }
}
