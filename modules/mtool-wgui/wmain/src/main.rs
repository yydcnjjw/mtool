fn main() {
    mapp::AppBuilder::new()
        .unwrap()
        .add_module_group(mtool_wgui::module())
        .add_module(mtool_cmder::Module::default())
        .add_module_group(mtool_interactive::module())
        .add_module_group(mtool_proxy::module())
        .add_module_group(mtool_translate::module())
        .build()
        .run();
}
