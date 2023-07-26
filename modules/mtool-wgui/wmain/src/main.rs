use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run() {
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

fn main() {}
