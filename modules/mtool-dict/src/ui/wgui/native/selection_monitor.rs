use mapp::{
    anyhow,
    prelude::*,
    sync::Mutex,
    tokio::{self, select},
    tokio_util::sync::CancellationToken,
    tracing::{debug, warn},
};
use mtool_main_window::wgui::native::{sticky, StickyWindow};
use mtool_system::event::{self, *};

use crate::dict::ecdict;

pub struct SelectionMonitor {
    token: Mutex<CancellationToken>,
    injector: Injector,
    ob: Res<event::Observer>,
}

impl SelectionMonitor {
    pub async fn construct(
        injector: Injector,
        ob: Res<event::Observer>,
    ) -> Result<Res<Self>, anyhow::Error> {
        Ok(Res::new(Self {
            token: Mutex::new(CancellationToken::new()),
            injector,
            ob,
        }))
    }

    pub fn cancel(&self) {
        self.token.lock().cancel();
    }

    pub fn run(&self) {
        let injector = self.injector.clone();
        let ob = self.ob.clone();

        let token = {
            let mut token = self.token.lock();
            *token = CancellationToken::new();
            token.clone()
        };

        tokio::spawn(async move {
            let mut rx = ob.subscribe();

            loop {
                select! {
                    _ = token.cancelled() => {
                        break
                    }
                    ev = rx.recv() => match ev {
                        Ok(ev) => {
                            if let Err(e) = Self::handle_system_event(ev, &injector).await {
                                warn!("{:?}", e);
                            }
                        },
                        Err(_) => {
                            break
                        },
                    }
                }
            }
        });
    }

    async fn handle_system_event(ev: Event, injector: &Injector) -> Result<(), anyhow::Error> {
        Ok(match ev {
            Event::Selection(ev) => Self::handle_selection_event(ev, injector).await?,
            _ => (),
        })
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
}

async fn dict_query_with_sticky(
    win: Res<StickyWindow>,
    dict: Res<ecdict::Dict>,
    query: String,
) -> Result<(), anyhow::Error> {
    debug!("{:?}", query);
    let result = dict.query(query.trim()).await?;
    sticky::show_main::<ecdict::DictView, _, _>(&win.base(), result).await?;
    win.show().await
}
