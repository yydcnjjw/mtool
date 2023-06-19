use std::future::Future;

use anyhow::Context;
use mapp::provider::Res;

use mtool_cmder::{Cmder, CreateCommandDescriptor};
use mtool_interactive::{Completion, CompletionArgs};
use notify_rust::{Notification, Timeout};

use super::{geosite_item::GeositeItem, ProxyService};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Operation has been cancelled")]
    OperationCancelled,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

async fn add_proxy_rule_inner(app: Res<ProxyService>, c: Res<Completion>) -> Result<(), Error> {
    let target = c
        .complete_read(
            CompletionArgs::<String>::without_completion()
                .prompt("Add proxy target: ")
                .hide_window(),
        )
        .await?
        .ok_or(Error::OperationCancelled)?;

    {
        let mut gs = app.resource.lock().unwrap();
        gs.insert_target("pri", &target)?;
        gs.store()?;
    }

    Ok(app.inner.router().add_rule_target(&app.proxy_id, &target)?)
}

async fn remove_proxy_rule_inner(app: Res<ProxyService>, c: Res<Completion>) -> Result<(), Error> {
    let target = {
        let items = {
            let gs = app.resource.lock().unwrap();
            if let Some(sg) = gs.get_site_group("pri") {
                sg.domain
                    .iter()
                    .cloned()
                    .map(|v| GeositeItem::new(v))
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
        .ok_or(Error::OperationCancelled)?
    };

    {
        let mut gs = app.resource.lock().unwrap();
        gs.remove_with_domain("pri", &target)?;
        gs.store()?;
    }
    Ok(())
}

async fn with_notify_result<F, O>(name: &str, f: F) -> Result<(), anyhow::Error>
where
    F: FnOnce() -> O,
    O: Future<Output = Result<(), Error>>,
{
    let mut notify = Notification::new();
    notify
        .appname("mtool proxy")
        .summary(name)
        .timeout(Timeout::Milliseconds(2000));

    match f().await {
        Ok(_) => {
            notify.body("successfully");
        }
        Err(e) => match e {
            Error::OperationCancelled => return Ok(()),
            Error::Other(e) => {
                notify.body(&format!("Error:\n{:?}", e));
            }
        },
    }

    notify.show().context("Failed to show notify")?;
    Ok(())
}

async fn add_proxy_rule(app: Res<ProxyService>, c: Res<Completion>) -> Result<(), anyhow::Error> {
    with_notify_result("add proxy rule", || async move {
        add_proxy_rule_inner(app, c).await
    })
    .await
}

async fn remove_proxy_rule(
    app: Res<ProxyService>,
    c: Res<Completion>,
) -> Result<(), anyhow::Error> {
    with_notify_result("remove proxy rule", || async move {
        remove_proxy_rule_inner(app, c).await
    })
    .await
}

pub async fn register(cmder: Res<Cmder>) -> Result<(), anyhow::Error> {
    cmder
        .add_command(add_proxy_rule.name("add_proxy_rule"))
        .add_command(
            remove_proxy_rule
                .name("remove_proxy_rule")
                .desc("remove proxy rule from file"),
        );

    Ok(())
}
