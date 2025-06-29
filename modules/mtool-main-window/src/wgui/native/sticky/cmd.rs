use mapp::{anyhow, prelude::*};
use mtool_cmder::{Cmder, CommandBuilder};

use super::window::StickyWindow;

async fn show_sticky_window(win: Res<StickyWindow>) -> Result<(), anyhow::Error> {
    win.show().await
}

async fn hide_sticky_window(win: Res<StickyWindow>) -> Result<(), anyhow::Error> {
    win.hide().await
}

pub async fn setup(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder
        .add_command(
            show_sticky_window
                .name("sticky-window.show")
                .descrption("Show sticky window"),
        )
        .add_command(
            hide_sticky_window
                .name("sticky-window.hide")
                .descrption("Hide sticky window"),
        );
    Ok(())
}
