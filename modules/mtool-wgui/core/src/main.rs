fn main() {
    mapp::AppBuilder::new().unwrap()
        .add_module_group(mtool_wgui_web::module())
        .build().run();
}
