use applications::{AppInfo, AppInfoContext};
use dioxus::prelude::*;

enum Command {
    Content(fn() -> Element),
    List(fn() -> Element),
}

fn register_command_palette_plugin() {}

struct View {}

async fn list_apps() -> Vec<applications::App> {
    let mut ctx = AppInfoContext::new(Vec::new());
    ctx.refresh_apps_in_background();

    ctx.get_all_apps()
}
