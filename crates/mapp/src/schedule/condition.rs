use crate::{context::ContextError, coroutine::Callable};

pub type BoxCondition = Box<dyn Callable<Output = bool, Error = ContextError>>;
