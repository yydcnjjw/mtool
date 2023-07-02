fn main() {
    mapp::AppBuilder::new()
        .unwrap()
        .add_module(mtool_wgui::module())
        .add_module(mtool_cmder::Module::default())
        .add_module(mtool_interactive::module())
        .add_module(mtool_proxy::module())
        .add_module(mtool_translate::module())
        .add_module(mtool_dict::module())
        .build()
        .run();
}
