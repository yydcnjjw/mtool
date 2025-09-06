use mapp::{anyhow, tokio::sync::oneshot};

pub(crate) enum Message {
    Task(
        (
            Box<dyn (FnOnce() -> Result<(), anyhow::Error>) + Send + Sync>,
            oneshot::Sender<Result<(), anyhow::Error>>,
        ),
    ),
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Task(_) => f.debug_tuple("Task").finish(),
        }
    }
}
