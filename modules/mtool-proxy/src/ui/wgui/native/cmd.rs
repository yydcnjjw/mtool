use mapp::{anyhow, prelude::*};
use mtool_main_window::wgui::native::{sticky, StickyWindow};

use crate::ui::wgui::web::View;

pub async fn show_stats(win: Res<StickyWindow>) -> Result<(), anyhow::Error> {
    sticky::show_main::<View, _, _>(&win.base(), ()).await?;
    win.show().await
}
