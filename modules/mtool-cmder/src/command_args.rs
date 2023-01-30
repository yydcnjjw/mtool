use std::ops::Deref;

pub struct CommandArgs {
    inner: Vec<String>,
}

impl Deref for CommandArgs {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl CommandArgs {
    pub fn new(inner: Vec<String>) -> Self {
        Self { inner }
    }
}
