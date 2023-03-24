fn main() {
    mapp::AppBuilder::new().unwrap()
        .add_module_group(mtool_wgui_core::module())
        .add_module_group(mtool_interactive_wgui::module())
        .build().run();
}
