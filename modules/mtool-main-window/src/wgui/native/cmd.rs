use mapp::prelude::*;
use mtool_cmder::{Cmder, CommandBuilder};

use super::{MtoolWindow, StickyWindow};

async fn hide_window(win: Res<MtoolWindow>) -> Result<(), anyhow::Error> {
    win.hide()?;
    Ok(())
}

async fn show_sticky_window(win: Res<StickyWindow>) -> Result<(), anyhow::Error> {
    win.show()?;
    Ok(())
}

async fn hide_sticky_window(win: Res<StickyWindow>) -> Result<(), anyhow::Error> {
    win.hide()?;
    Ok(())
}

pub async fn init(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder
        .add_command(
            hide_window
                .name("main-window.hide")
                .descrption("Hide main window"),
        )
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
