use mapp::{anyhow, serde::Serialize};
use mtool_wgui::generic::Props;
use std::{any::type_name, ops::Deref, path::PathBuf};
use yew::prelude::*;

pub trait CompleteItem: Serialize + Clone + Send + Sync + 'static {
    type WGuiView: BaseComponent<Message = ()>;

    fn complete_hint(&self) -> String;

    fn try_from_completed(_completed: &str) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        Err(anyhow::anyhow!(
            "TryFrom of {} is not implemented",
            type_name::<Self>()
        ))
    }
}

impl CompleteItem for String {
    type WGuiView = TextCompleteItemView;

    fn complete_hint(&self) -> String {
        self.to_string()
    }

    fn try_from_completed(completed: &str) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        Ok(completed.to_string())
    }
}

#[function_component]
pub fn TextCompleteItemView(props: &Props<String>) -> Html {
    html! {
        <div class={classes!(
        )}> { props.deref().clone() }
        </div>
    }
}

impl CompleteItem for PathBuf {
    type WGuiView = PathBufCompleteItemView;

    fn complete_hint(&self) -> String {
        self.display().to_string()
    }

    fn try_from_completed(completed: &str) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        Ok(PathBuf::try_from(completed)?)
    }
}

#[function_component]
pub fn PathBufCompleteItemView(props: &Props<PathBuf>) -> Html {
    html! {
        <div class={classes!(
        )}> { props.display().to_string() } </div>
    }
}
