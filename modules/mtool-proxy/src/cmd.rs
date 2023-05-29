use anyhow::Context;
use mapp::provider::Res;
use mproxy::{protos, router::parse_target};
use mtool_interactive::{Completion, CompletionArgs};
use notify_rust::{Notification, Timeout};

use crate::proxy::ProxyApp;

async fn add_proxy_rule_inner(app: Res<ProxyApp>, c: Res<Completion>) -> Result<(), anyhow::Error> {
    let target = c
        .complete_read(
            CompletionArgs::without_completion()
                .prompt("Add proxy target: ")
                .hide_window(),
        )
        .await?;

    {
        let mut gs = app.resource.lock().unwrap();
        let (rule_type, value) = parse_target(&target)?;
        gs.insert("pri", rule_type, value)?;
        gs.store()?;
    }

    app.inner.router().add_rule_target(&app.proxy_id, &target)
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
                    .map(|item| -> String {
                        format!(
                            "{}: {}",
                            if let Ok(type_) = item.type_.enum_value() {
                                match type_ {
                                    protos::geosite::domain::Type::Domain => "Domain",
                                    protos::geosite::domain::Type::Plain => "Plain",
                                    protos::geosite::domain::Type::Regex => "Regex",
                                    protos::geosite::domain::Type::Full => "Full",
                                }
                            } else {
                                "Unknown"
                            },
                            item.value
                        )
                    })
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
        let (rule_type, value) = parse_target(&target)?;
        gs.remove("pri", rule_type, value)?;
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
