#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[cfg(all(target_os = "android", feature = "mobile"))]
mod android;

fn run(mut builder: mapp::AppBuilder) {
    builder
        .add_module(mtool_core::module())
        .add_module(mtool_rpc::module())
        .add_module(mtool_storage::module());

    #[cfg(feature = "graphic")]
    builder
        .add_module(mtool_dioxus::module())
        .add_module(mtool_system::module());

    #[cfg(any(feature = "mobile", feature = "desktop"))]
    builder.add_module(mtool_assistant::module());

    #[cfg(feature = "desktop")]
    builder
        .add_module(mtool_cmdpal::module())
        .add_module(mtool_proxy::module())
        .add_module(mtool_apps::module());

    builder.build().run();
}

fn main() {
    run(mapp::AppBuilder::new().unwrap());
}
