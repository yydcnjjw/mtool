use std::{
    hash::{Hash, Hasher},
    ops::Deref,
};

use anyhow::Context;
use mapp::provider::Res;
use mproxy::protos::geosite;
use mtool_interactive::{CompleteItem, Completion, CompletionArgs};
use notify_rust::{Notification, Timeout};
use yew::prelude::*;

use crate::proxy::ProxyApp;

async fn add_proxy_rule_inner(app: Res<ProxyApp>, c: Res<Completion>) -> Result<(), anyhow::Error> {
    let target = c
        .complete_read(
            CompletionArgs::<String>::without_completion()
                .prompt("Add proxy target: ")
                .hide_window(),
        )
        .await?;

    {
        let mut gs = app.resource.lock().unwrap();
        gs.insert_target("pri", &target)?;
        gs.store()?;
    }

    app.inner.router().add_rule_target(&app.proxy_id, &target)
}

#[derive(Properties, PartialEq, Clone)]
struct GeositeItem {
    data: geosite::Domain,
}

impl Hash for GeositeItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.data.type_.enum_value() {
            Ok(v) => v.hash(state),
            Err(_) => {}
        }
        self.data.value.hash(state);
    }
}

impl Eq for GeositeItem {}

impl Deref for GeositeItem {
    type Target = geosite::Domain;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl CompleteItem for GeositeItem {
    type WGuiView = GeositeItemView;

    fn complete_hint(&self) -> String {
        self.data.value.clone()
    }
}

struct GeositeItemView;
impl Component for GeositeItemView {
    type Message = ();

    type Properties = GeositeItem;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let props = ctx.props();
        let type_ = match props.type_.enum_value() {
            Ok(v) => match v {
                geosite::domain::Type::Plain => "Plain",
                geosite::domain::Type::Regex => "Regex",
                geosite::domain::Type::Domain => "Domain",
                geosite::domain::Type::Full => "Full",
            },
            Err(_) => "Unknown",
        };
        html! {
            <div>
                <span>
                  { type_ }
                </span>
                {": "}
                <span>
                  { props.value.clone() }
                </span>
            </div>
        }
    }
}

async fn remove_proxy_rule_inner(
    app: Res<ProxyApp>,
    c: Res<Completion>,
) -> Result<(), anyhow::Error> {
    let target = {
        let items = {
            let gs = app.resource.lock().unwrap();
            if let Some(sg) = gs.get_site_group("pri") {
                sg.domain
                    .iter()
                    .cloned()
                    .map(|v| GeositeItem { data: v })
                    .collect::<Vec<_>>()
            } else {
                vec![]
            }
        };

        c.complete_read(
            CompletionArgs::with_vec(items)
                .prompt("Remove proxy target: ")
                .hide_window(),
        )
        .await?
    };

    {
        let mut gs = app.resource.lock().unwrap();
        gs.remove_with_domain("pri", &target)?;
        gs.store()?;
    }
    Ok(())
}

pub async fn add_proxy_rule(app: Res<ProxyApp>, c: Res<Completion>) -> Result<(), anyhow::Error> {
    let mut notify = Notification::new();
    notify
        .appname("mtool proxy")
        .summary("add proxy rule")
        .timeout(Timeout::Milliseconds(2000));

    match add_proxy_rule_inner(app, c).await {
        Ok(_) => {
            notify.body("successfully");
        }
        Err(e) => {
            notify.body(&format!("Error:\n{:?}", e));
        }
    }

    notify.show().context("Failed to show notify")?;
    Ok(())
}

pub async fn remove_proxy_rule(
    app: Res<ProxyApp>,
    c: Res<Completion>,
) -> Result<(), anyhow::Error> {
    let mut notify = Notification::new();
    notify
        .appname("mtool proxy")
        .summary("remove proxy rule")
        .timeout(Timeout::Milliseconds(2000));

    match remove_proxy_rule_inner(app, c).await {
        Ok(_) => {
            notify.body("successfully");
        }
        Err(e) => {
            notify.body(&format!("Error:\n{:?}", e));
        }
    }

    notify.show().context("Failed to show notify")?;
    Ok(())
}
