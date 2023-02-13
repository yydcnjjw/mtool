use std::sync::Arc;

use crate::{
    module::{Module, ModuleGroup},
    provider::Injector,
    Schedule,
};

pub struct AppBuilder {
    modules: ModuleGroup,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            modules: Default::default(),
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
        let builder = Arc::new(std::mem::replace(self, AppBuilder::new()));

        let mut app_runner = AppRunner::new();

        app_runner.runner = Some(Box::new(move || {
            #[cfg(feature = "tracing")]
            console_subscriber::init();

            let mut ctx = AppContext::new();
            let builder = builder.clone();

            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async move {
                    builder.modules.init(&mut ctx).await.unwrap();

                    let sche = ctx.schedule;

                    let mut app = App::new();
                    app.injector = ctx.injector;

                    sche.run(&app).await.unwrap();
                });
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
    runner: Option<Box<dyn Fn()>>,
}

impl AppRunner {
    pub fn new() -> Self {
        Self { runner: None }
    }

    pub fn run(self) {
        (self.runner.unwrap())();
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
