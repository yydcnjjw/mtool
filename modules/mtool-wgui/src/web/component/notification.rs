use super::ProgressBar;
use mtauri_sys::{
    event::Event,
    window::{UnlistenFn, Window},
};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use yew::{prelude::*, suspense::use_future};

#[derive(Debug, Clone, Serialize, Deserialize, Properties, PartialEq)]
pub struct ProgressNotificationProps {
    pub id: String,
    pub message: String,
    pub progress: usize,
}

async fn listen_progress_notification<UpdateFn>(
    id: String,
    update: UpdateFn,
) -> Result<impl UnlistenFn, anyhow::Error>
where
    UpdateFn: Fn(ProgressNotificationProps) + 'static,
{
    Window::current()?
        .listen(
            "update_progress_notification",
            move |ev: Event<ProgressNotificationProps>| {
                if ev.payload.id == id {
                    update(ev.payload);
                }
                Ok(())
            },
        )
        .await
}

#[function_component(ProgressNotification)]
pub fn progress_notification(props: &ProgressNotificationProps) -> HtmlResult {
    let progress = use_state(|| props.progress);
    let message = use_state(|| props.message.clone());

    {
        let progress = progress.clone();
        let message = message.clone();
        let unlisten =
            use_future(move || listen_progress_notification(props.id.clone(), move |props| {
                progress.set(props.progress);
                message.set(props.message);
            }))?;
        use_effect_with((), move |_| {
            move || {
                let _ = match unlisten.deref() {
                    Ok(unlisten) => unlisten(),
                    Err(_) => Ok({}),
                };
            }
        });
    }

    Ok(html! {
    <div>
      <div>{(*message).clone()}</div>
      <ProgressBar progress={*progress}/>
    </div>
    })
}
