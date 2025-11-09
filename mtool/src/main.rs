#[cfg(all(target_os = "android", feature = "mobile"))]
mod android;

fn run(mut builder: mapp::AppBuilder) {
    builder
        .add_module(mtool_core::module())
        .add_module(mtool_p2p::module())
        .add_module(mtool_storage::module());

    #[cfg(feature = "system")]
    builder
        .add_module(mtool_system::module())
        .add_module(mtool_emacs::module());

    #[cfg(feature = "graphic")]
    builder.add_module(mtool_dioxus::module());

    #[cfg(feature = "desktop")]
    builder
        .add_module(mtool_cmdpal::module())
        .add_module(mtool_proxy::module())
        .add_module(mtool_apps::module());

    #[cfg(any(feature = "mobile", feature = "desktop"))]
    builder.add_module(mtool_assistant::module());

    builder.build().run();
}

fn main() {
    run(mapp::AppBuilder::new().unwrap());
}
