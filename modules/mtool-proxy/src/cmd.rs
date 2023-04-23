use anyhow::Context;
use mapp::provider::Res;
use mproxy::router::parse_target;
use mtool_interactive::{Completion, CompletionArgs};
use notify_rust::{Notification, Timeout};

use crate::proxy::ProxyApp;

async fn add_proxy_rule_inner(app: Res<ProxyApp>, c: Res<Completion>) -> Result<(), anyhow::Error> {
    let target = c
        .complete_read(CompletionArgs::without_completion().prompt("Add proxy target: "))
        .await?;

    {
        let mut gs = app.resource.lock().unwrap();
        let (rule_type, value) = parse_target(&target)?;
        gs.insert("pri", rule_type, value)?;
        gs.store()?;
    }

    app.inner.router().add_rule_target(&app.proxy_id, &target)
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
