fn main() {
    mapp::AppBuilder::new().unwrap()
        .add_module_group(mtool_core::module())
        .add_module_group(mtool_system::module())
        .add_module_group(mtool_wgui::module())
        .add_module(mtool_cmder::Module::default())
        .add_module(mtool_translate::Module::default())
        .add_module(mtool_dict::Module::default())
        .add_module(mtool_interactive::module())
        .add_module(mtool_proxy::Module::default())
        .build()
        .run();
}
