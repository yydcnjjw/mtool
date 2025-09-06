use copypasta::{ClipboardContext, ClipboardProvider};
use dioxus::{core::use_hook_with_cleanup, prelude::*};
use mapp::{
    anyhow::{self, anyhow, Context},
    cfg_if::cfg_if,
    itertools::Itertools,
    notify_rust::{Notification, Timeout},
    prelude::*,
    reqwest::Url,
    tracing::warn,
};
use mproxy::protos::geosite;
use mtool_cmdpal::{
    components::{CommandInput, CommandList},
    Command, CommandItem, CommandPalette, CommandResult, InvokeCommand,
};
use mtool_dioxus::{generate_keymap, local_action, prelude::*};
use std::sync::Arc;

use crate::proxy_service::ProxyService;

async fn notify_result<T>(name: &str, result: Result<T, anyhow::Error>) {
    let mut notify = Notification::new();
    notify
        .appname("mtool proxy")
        .summary(name)
        .timeout(Timeout::Milliseconds(2000));

    match result {
        Ok(_) => notify.body("successfully"),
        Err(e) => notify.body(&format!("Error:\n{:?}", e)),
    };

    if let Err(e) = {
        cfg_if! {
            if #[cfg(windows)] {
                notify.show().context("Failed to show notify")
            } else if #[cfg(unix)] {
                notify.show_async().await.context("Failed to show notify")
            } else {
                unimplemented!()
            }
        }
    } {
        warn!("{:?}", e);
    }
}

fn clipboard_content() -> Result<String, anyhow::Error> {
    let mut ctx = ClipboardContext::new().map_err(|e| anyhow!("{:?}", e))?;
    ctx.get_contents().map_err(|e| anyhow!("{:?}", e))
}

#[component]
fn AddProxyRuleView() -> Element {
    let service = use_app_context::<Res<ProxyService>>().suspend()?;

    let keybinding = use_context::<Keybinding>();

    let mut command_input: Signal<CommandInput> = use_context();
    let mut command_result: Signal<CommandResult> = use_context();

    use_hook(|| {
        if let Some(target) = clipboard_content()
            .ok()
            .and_then(|content| Url::parse(&content).ok())
            .and_then(|url| url.domain().map(|domain| domain.to_owned()))
        {
            command_input.set(format!("d:{}", target).into());
        }
    });

    let add_proxy_rule = use_callback(move |_| -> Result<(), anyhow::Error> {
        to_owned![service];
        spawn(async move {
            let target = command_input().value;
            notify_result(
                &format!("Add proxy rule: {}", &target),
                service().add_routing_rule(&target).await,
            )
            .await;
            command_result.set(CommandResult::Dismiss);
        });
        Ok(())
    });

    use_hook_with_cleanup(
        move || {
            let name = format!("proxy.add_proxy_rule");
            let km = generate_keymap!(("<Return>", local_action!(add_proxy_rule)),).unwrap();

            keybinding.push_keymap(&name, km);
            (name, keybinding)
        },
        move |(name, keybinding)| {
            keybinding.remove_keymap(&name);
        },
    );

    rsx! {}
}

pub struct RemoveProxyRuleCommand {
    service: Res<ProxyService>,
    domain: geosite::Domain,
}

impl RemoveProxyRuleCommand {
    pub fn new(service: Res<ProxyService>, domain: geosite::Domain) -> Self {
        Self { service, domain }
    }
}

#[async_trait]
impl InvokeCommand for RemoveProxyRuleCommand {
    async fn invoke(&self) -> Result<CommandResult, anyhow::Error> {
        notify_result(
            &format!("Remove proxy rule: {}", self.domain.value),
            self.service.remove_routing_rule(&self.domain).await,
        )
        .await;
        Ok(CommandResult::Dismiss)
    }
}

#[component]
fn RemoveProxyRuleView() -> Element {
    let service = use_app_context::<Res<ProxyService>>().suspend()?;

    let items = use_hook(move || {
        service()
            .routing_rules()
            .into_iter()
            .map(|item| CommandItem {
                title: item.value.clone(),
                sub_title: "Domain".to_owned(),
                icon: None,
                hints: vec![item.value.clone()],
                source: "proxy".to_owned(),
                command: Arc::new(Command::new(
                    "proxy.remove_proxy_rule",
                    RemoveProxyRuleCommand::new(service(), item.clone()),
                )),
                more_commands: Vec::new(),
            })
            .collect_vec()
    });

    rsx! {
        CommandList { items }
    }
}

async fn add_proxy_rule() -> Result<CommandResult, anyhow::Error> {
    Ok(CommandResult::ShowView(|| rsx! { AddProxyRuleView {} }))
}

async fn remove_proxy_rule() -> Result<CommandResult, anyhow::Error> {
    Ok(CommandResult::ShowView(|| rsx! { RemoveProxyRuleView {} }))
}

pub async fn register(cmdpal: Res<CommandPalette>) -> Result<(), anyhow::Error> {
    cmdpal
        .add_top_level_command(CommandItem::from(
            Command::new("Add proxy rule", add_proxy_rule).description("Add proxy rule to mproxy"),
        ))
        .add_top_level_command(CommandItem::from(
            Command::new("Remove proxy rule", remove_proxy_rule)
                .description("Remove proxy rule to mproxy"),
        ));

    Ok(())
}
