#[cfg(target_os = "android")]
mod android;

pub fn run(mut builder: mapp::AppBuilder) {
    builder
        .add_module(mtool_core::module())
        .add_module(mtool_system::module())
        .add_module(mtool_storage::module())
        .add_module(mtool_dioxus::module())
        .add_module(mtool_cmdpal::module())
        // .add_module(mtool_assistant::module())
        // .add_module(mtool_proxy::module())
        ;

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    builder.add_module(mtool_apps::module());

    // .add_module(mtool_pdf::module())
    // .add_module(mtool_interactive::module())
    // .add_module(mtool_wgui::module::<tauri::Wry>(tauri::generate_context!()))
    // .add_module(mtool_main_window::module())
    // .add_module(mtool_cmder::Module::default())
    // .add_module(mtool_translate::module())
    // .add_module(mtool_dict::module())
    // .add_module(mtool_interactive::module())
    // .add_module(mtool_proxy::module())
    // .add_module(mtool_toast::Module::default())
    // .add_module(mtool_ai::Module::default())
    // .add_module(mtool_pdf::module())
    builder.build().run();
}
