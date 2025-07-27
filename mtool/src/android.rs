use mapp::{android_activity::AndroidApp, anyhow, prelude::*};

use crate::run;

struct Module {
    android_app: AndroidApp,
}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().insert(Res::new(self.android_app.clone()));
        Ok(())
    }
}

#[no_mangle]
fn android_main(android_app: AndroidApp) {
    let mut builder = mapp::AppBuilder::new().unwrap();

    builder.add_module(Module { android_app });

    run(builder);
}
