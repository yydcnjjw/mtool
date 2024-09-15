use mapp::{anyhow, prelude::*};
use mtool_cmder::{Cmder, CommandBuilder};

use super::window::MtoolWindow;

async fn hide_window(win: Res<MtoolWindow>) -> Result<(), anyhow::Error> {
    win.hide().await
}

pub async fn setup(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder.add_command(
        hide_window
            .name("main-window.hide")
            .descrption("Hide main window"),
    );

    Ok(())
}
