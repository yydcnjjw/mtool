use emacs::{defun, Env};
use mapp::prelude::*;
use mtool_system::{
    AppInfo, Notification, NotificationContent, RemoteSystemEvent, RemoteSystemEventSource,
    SystemEvent::NotificationPosted,
};

use crate::context::EmacsContext;

#[defun(mod_in_name = false)]
fn alert(
    _env: &Env,
    ctx: &EmacsContext,
    message: String,
    title: Option<String>,
    category: Option<String>,
    id: Option<String>,
    icon: Option<String>,
) -> Result<(), emacs::Error> {
    let app = AppInfo { id: "emacs".into() };
    let content = NotificationContent {
        message,
        title,
        icon,
        id,
    };
    let notification = match category {
        Some(category) if category == "agenda" => Notification::Agenda { app, content },
        _ => Notification::Generic { app, content },
    };

    ctx.spawn(
        move |system_source: Res<RemoteSystemEventSource>| async move {
            system_source
                .publish(&RemoteSystemEvent {
                    source: "emacs".into(),
                    event: NotificationPosted(notification),
                })
                .await
        },
    );

    Ok(())
}
