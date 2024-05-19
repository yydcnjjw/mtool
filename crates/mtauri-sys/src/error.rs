use std::fmt;

pub trait IntoAnyhowError<O> {
    fn into_anyhow(self) -> Result<O, anyhow::Error>;
}

impl<O, E> IntoAnyhowError<O> for Result<O, E>
where
    E: fmt::Debug,
{
    fn into_anyhow(self) -> Result<O, anyhow::Error> {
        self.map_err(|e| anyhow::anyhow!("{:?}", e))
    }
}
