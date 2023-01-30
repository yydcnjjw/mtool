use std::future::Future;

use async_trait::async_trait;
use mtool_interactive_model::CompletionMeta;

#[async_trait]
pub trait CompleteRead {
    async fn complete_read(&self, args: CompletionArgs) -> Result<String, anyhow::Error>;
}

pub struct CompletionArgs {
    pub complete: Box<dyn Complete + Send + Sync>,
    pub meta: CompletionMeta,
}

impl CompletionArgs {
    pub fn new<C>(c: C) -> Self
    where
        C: Complete + Send + Sync + 'static,
    {
        Self {
            complete: Box::new(c),
            meta: CompletionMeta::default(),
        }
    }

    pub fn prompt(mut self, prompt: &str) -> Self {
        self.meta.prompt = prompt.to_string();
        self
    }

    pub async fn complete(&self, completed: String) -> Result<Vec<String>, anyhow::Error> {
        self.complete.complete(completed).await
    }
}

#[async_trait]
pub trait Complete {
    async fn complete(&self, completed: String) -> Result<Vec<String>, anyhow::Error>;
}

#[async_trait]
impl<Func, Output> Complete for Func
where
    Func: Fn(String) -> Output + Send + Sync,
    Output: Future<Output = Result<Vec<String>, anyhow::Error>> + Send,
{
    async fn complete(&self, completed: String) -> Result<Vec<String>, anyhow::Error> {
        (self)(completed).await
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
                args.complete.complete(args.prompt).await?;
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
