use std::{any::type_name, fmt::Display, future::Future};

use async_trait::async_trait;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use itertools::Itertools;
use mtool_interactive_model::CompletionMeta;
use yew::prelude::*;

use crate::utils::rand_string;

#[derive(Properties, Clone, PartialEq)]
pub struct Props<T>
where
    T: PartialEq,
{
    pub data: T,
}

impl<T> Props<T>
where
    T: PartialEq,
{
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl<T: Display> Display for Props<T>
where
    T: PartialEq,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

pub trait CompleteItem:
    PartialEq
    + Clone
    + Into<<<Self as CompleteItem>::WGuiView as Component>::Properties>
    + TryFromCompleted
    + Send
    + Sync
    + 'static
{
    type WGuiView: Component;
    fn complete_hint(&self) -> String;
}

pub trait TryFromCompleted {
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

#[async_trait]
pub trait CompleteRead {
    async fn complete_read<T>(&self, args: CompletionArgs<T>) -> Result<T, anyhow::Error>
    where
        T: CompleteItem;
}

#[async_trait]
pub trait Complete<T>
where
    T: CompleteItem,
{
    async fn complete(&self, completed: &str) -> Result<Vec<T>, anyhow::Error>;
}

#[async_trait]
impl<T, Func, Output> Complete<T> for Func
where
    Func: Fn(&str) -> Output + Send + Sync,
    Output: Future<Output = Result<Vec<T>, anyhow::Error>> + Send,
    T: CompleteItem,
{
    async fn complete(&self, completed: &str) -> Result<Vec<T>, anyhow::Error> {
        (self)(completed).await
    }
}

pub struct CompletionArgs<T>
where
    T: CompleteItem,
{
    complete: Box<dyn Complete<T> + Send + Sync>,
    completion_meta: CompletionMeta,
    hide_window: bool,
}

impl<T> CompletionArgs<T>
where
    T: CompleteItem,
{
    pub fn new<C>(c: C) -> Self
    where
        C: Complete<T> + Send + Sync + 'static,
    {
        Self {
            complete: Box::new(c),
            completion_meta: CompletionMeta {
                id: rand_string(),
                prompt: String::default(),
            },
            hide_window: false,
        }
    }

    pub fn without_completion() -> Self {
        Self::new(|_: &str| async move { Ok(Vec::new()) })
    }

    pub fn with_vec(items: Vec<T>) -> Self
    where
        T: 'static,
    {
        Self::new(CompleteVec::new(items))
    }

    pub fn prompt(mut self, prompt: &str) -> Self {
        self.completion_meta.prompt = prompt.to_string();
        self
    }

    pub fn hide_window(mut self) -> Self {
        self.hide_window = true;
        self
    }

    pub fn need_hide_window(&self) -> bool {
        self.hide_window
    }

    pub fn completion_meta(&self) -> &CompletionMeta {
        &self.completion_meta
    }
}

#[async_trait]
impl<T> Complete<T> for CompletionArgs<T>
where
    T: CompleteItem,
{
    async fn complete(&self, completed: &str) -> Result<Vec<T>, anyhow::Error> {
        self.complete.complete(completed).await
    }
}

pub struct CompleteVec<T> {
    matcher: SkimMatcherV2,
    items: Vec<T>,
}

impl<T> CompleteVec<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
            items,
        }
    }
}

#[async_trait]
impl<T> Complete<T> for CompleteVec<T>
where
    T: CompleteItem + Clone,
{
    async fn complete(&self, completed: &str) -> Result<Vec<T>, anyhow::Error> {
        let mut items = Vec::new();

        for item in self.items.iter() {
            let hint = item.complete_hint();
            if let Some(score) = self.matcher.fuzzy_match(&hint, &completed) {
                items.push((score, hint, item.clone()));
            }
        }

        items.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(items
            .into_iter()
            .unique_by(|v| v.1.clone())
            .map(|item| item.2)
            .collect_vec())
    }
}

impl CompleteItem for String {
    type WGuiView = TextCompleteItemView;

    fn complete_hint(&self) -> String {
        self.to_string()
    }
}

impl TryFromCompleted for String {
    fn try_from_completed(completed: &str) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        Ok(completed.to_string())
    }
}

pub struct TextCompleteItemView;

impl From<String> for Props<String> {
    fn from(value: String) -> Self {
        Props::new(value)
    }
}

impl Component for TextCompleteItemView {
    type Message = ();

    type Properties = Props<String>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div> { ctx.props().data.clone() } </div>
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_complete() {
        struct TestCompleteRead {}
        #[async_trait]
        impl CompleteRead for TestCompleteRead {
            async fn complete_read(&self, args: CompletionArgs) -> Result<String, anyhow::Error> {
                args.complete.complete(args.completion_meta.prompt).await?;
                Ok(String::default())
            }
        }

        TestCompleteRead {}
            .complete_read(
                CompletionArgs::new(|_completed: String| async move {
                    Ok::<Vec<String>, anyhow::Error>(Vec::new())
                })
                .prompt("test"),
            )
            .await
            .unwrap();
    }
}
