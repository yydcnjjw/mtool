fn main() {
    mtool_core::logger::early_init();

    mapp::AppBuilder::new()
        .add_module_group(mtool_core::module())
        .add_module_group(mtool_system::module())
        .add_module(mtool_cmder::Module::default())
        .add_module(mtool_toast::Module::default())
        .add_module(mtool_translate::Module::default())
        .add_module(mtool_dict::Module::default())
        .build()
        .run();
}
