use std::future::Future;
use async_trait::async_trait;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::rand::rand_string;

use super::complete_item::CompleteItem;

#[async_trait]
pub trait CompleteRead {
    async fn complete_read<T>(&self, args: CompletionArgs<T>) -> Result<Option<T>, anyhow::Error>
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

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CompletionMeta {
    pub id: String,
    pub prompt: String,
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
