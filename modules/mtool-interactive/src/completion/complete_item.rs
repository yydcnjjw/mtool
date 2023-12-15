use serde::{Deserialize, Deserializer, Serialize};
use std::{any::type_name, fmt::Display, ops::Deref, path::PathBuf};
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

#[derive(Properties, Clone, PartialEq, Serialize)]
pub struct Props<T>
where
    T: PartialEq + Serialize,
{
    data: T,
}

impl<'a, T> Deserialize<'a> for Props<T>
where
    T: PartialEq + Serialize + Deserialize<'a>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        Ok(Props::new(T::deserialize(deserializer)?))
    }
}

impl<T> Props<T>
where
    T: PartialEq + Serialize,
{
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl<T> Deref for Props<T>
where
    T: PartialEq + Serialize,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> From<T> for Props<T>
where
    T: PartialEq + Serialize,
{
    fn from(value: T) -> Self {
        Props::new(value)
    }
}

impl<T: Display> Display for Props<T>
where
    T: PartialEq + Serialize,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
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
