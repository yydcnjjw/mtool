use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

#[async_trait]
pub trait Command: Send + Sync + Debug {
    async fn exec(&mut self, args: Vec<String>);
}

pub type AsyncCommand = Arc<Mutex<dyn Command>>;

pub struct ClosureCmd<Closure> {
    closure: Closure,
}

impl<Closure> Debug for ClosureCmd<Closure> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClosureCmd").finish()
    }
}

impl<Closure> ClosureCmd<Closure> {
    pub fn new(closure: Closure) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self { closure }))
    }
}

#[async_trait]
impl<Closure> Command for ClosureCmd<Closure>
where
    Closure: FnMut(Vec<String>) + Send + Sync,
{
    async fn exec(&mut self, args: Vec<String>) {
        (self.closure)(args);
    }
}
