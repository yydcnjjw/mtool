use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("provider for {value} not found"))]
    ProvideNotFound { value: &'static str },
}
