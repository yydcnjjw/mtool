use std::panic;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    
    mapp::LocalAppBuilder::new()
        .unwrap()
        .add_module(mtool_wgui::web_module())
        .add_module(mtool_cmder::web_module())
        .add_module(mtool_interactive::web_module())
        .add_module(mtool_proxy::web_module())
        .add_module(mtool_translate::web_module())
        .add_module(mtool_dict::web_module())
        .add_module(mtool_pdf::web_module())
        .build()
        .run();    
}
