use std::{
    env,
    ffi::CStr,
    fs::{self, File},
    io::{self, Read},
    os,
    path::PathBuf,
};

use mapp::{
    android_activity::AndroidApp,
    anyhow::{self, Context},
    itertools::Itertools,
    prelude::*,
    tracing::{info, warn},
};
use mtool_dioxus::desktop::wry::prelude::ndk::asset::{AssetDir, AssetManager};

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
fn android_main(app: AndroidApp) {
    let mut builder = mapp::AppBuilder::new().unwrap();

    builder.add_module(Module { android_app: app });

    run(builder);
}
