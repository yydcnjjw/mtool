use mapp::provider::Res;
use mproxy::router::parse_target;
use mtool_interactive::{Completion, CompletionArgs};

use crate::proxy::ProxyApp;

pub async fn add_proxy_target(app: Res<ProxyApp>, c: Res<Completion>) -> Result<(), anyhow::Error> {
    let target = c
        .complete_read(CompletionArgs::without_completion().prompt("Add proxy target: "))
        .await?;

    {
        let mut gs = app.resource.lock().unwrap();
        let (rule_type, value) = parse_target(&target)?;
        gs.insert("pri", rule_type, value)?;
        gs.store()?;
    }

    app.inner
        .router()
        .add_rule_target(&app.proxy_id, &target)
}
