use tracing::{debug, error};

use crate::{
    module::{Module, ModuleGroup},
    provider::{Injector, Res},
    tracing::Tracing,
    Schedule,
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
        Mod: Module + Send + Sync + 'static,
    {
        self.modules.add_module(module);
        self
    }

    pub fn add_module_group(&mut self, modules: ModuleGroup) -> &mut Self {
        self.modules.add_module_group(modules);
        self
    }

    pub fn build(&mut self) -> AppRunner {
        let builder = std::mem::replace(self, AppBuilder::empty());

        let mut app_runner = AppRunner::new();

        app_runner.runner = Some(Box::new(move || -> Result<(), anyhow::Error> {
            let mut ctx = AppContext::new();

            ctx.injector().insert(Res::new(builder.tracing.unwrap()));

            let modules = builder.modules;

            #[cfg(target_arch = "wasm32")]
            let mut rt = tokio::runtime::Builder::new_current_thread();

            #[cfg(not(target_arch = "wasm32"))]
            let mut rt = tokio::runtime::Builder::new_multi_thread();

            rt.enable_all().build()?.block_on(async move {
                modules.init(&mut ctx).await?;

                let sche = ctx.schedule;

                let mut app = App::new();
                app.injector = ctx.injector;

                sche.run(&app).await?;

                debug!("App running!");
                Ok::<(), anyhow::Error>(())
            })
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
            eprintln!("{:?}", e);
        }
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

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use crate::{define_label, provider::Res, Label};

    use super::*;

    struct TestModule {}

    define_label!(
        enum TestStage {
            PreTest,
            Test,
            PostTest,
        }
    );

    #[async_trait]
    impl Module for TestModule {
        async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
            app.injector().insert(Res::new(10i32));
            app.injector().insert(Res::new(String::from("test")));

            app.schedule()
                .add_stage(TestStage::PreTest)
                .add_stage(TestStage::Test)
                .add_stage(TestStage::PostTest)
                .add_task(
                    TestStage::Test,
                    |v1: Res<i32>, v2: Res<String>| async move {
                        println!("test module {}, {}", *v1, *v2);
                        Ok(())
                    },
                );

            println!("test module init");
            Ok(())
        }
    }

    #[test]
    fn test_app() {
        AppBuilder::new().add_module(TestModule {}).build().run();
    }
}
