mod cmd;
mod plugin;

use mapp::{inject::inject_once, prelude::*};
use mtool_cmder::{Cmder, CommandBuilder};
use mtool_main_window::wgui::native::sticky::{api::show_sub_view, window::StickyWindow};
use mtool_system::event::{self, Event, SelectionEvent, PLAIN, TEXT};
use mtool_wgui::WGuiStage;
use tracing::{debug, warn};

use crate::dict::ecdict;

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.schedule()
            .add_once_task(WGuiStage::Setup, plugin::setup)
            .add_once_task(WGuiStage::AfterInit, init);
        Ok(())
    }
}

async fn init(
    cmder: Res<Cmder>,
    ob: Res<event::Observer>,
    injector: Injector,
) -> Result<(), anyhow::Error> {
    cmder
        .add_command(
            cmd::query_dict_with_clipboard
                .name("dict.query_with_clipboard")
                .descrption("Query dict with clipboard"),
        )
        .add_command(cmd::query_dict.name("dict.query").descrption("Query dict"));

    tokio::spawn(async move {
        let mut rx = ob.subscribe();
        while let Ok(ev) = rx.recv().await {
            if let Err(e) = handle_system_event(ev, &injector).await {
                warn!("{:?}", e);
            }
        }
    });

    Ok(())
}

async fn handle_system_event(ev: Event, injector: &Injector) -> Result<(), anyhow::Error> {
    if let Err(e) = match ev {
        Event::Selection(ev) => handle_selection_event(ev, injector).await,
        _ => Ok(()),
    } {
        warn!("{:?}", e);
    }
    Ok(())
}
async fn handle_selection_event(
    ev: SelectionEvent,
    injector: &Injector,
) -> Result<(), anyhow::Error> {
    let SelectionEvent { data, mime_type } = ev;
    match (mime_type.type_(), mime_type.subtype()) {
        (TEXT, PLAIN) => {
            if let Ok(text) = String::from_utf8(data) {
                if !text.trim().contains(" ") {
                    inject_once(injector, move |win, dict| {
                        dict_query_with_sticky(win, dict, text)
                    })
                    .await?
                    .await?;
                }
            }
        }
        (TEXT, _) => {}
        _ => {}
    }

    Ok(())
}

async fn dict_query_with_sticky(
    win: Res<StickyWindow>,
    dict: Res<ecdict::Dict>,
    query: String,
) -> Result<(), anyhow::Error> {
    debug!("{:?}", query);
    let result = dict.query(query.trim()).await?;
    show_sub_view::<ecdict::DictView, _, _>(win.clone(), "dict", result).await?;
    win.show()?;
    Ok(())
}
