use std::rc::Rc;

use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum ContextError {
    #[snafu(display("provider for {value} not found"))]
    ProvideNotFound { value: &'static str },

    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
        source: Option<Box<dyn std::error::Error>>,
    },
}

#[derive(Debug, Clone, Snafu)]
pub enum ProvideError {
    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Rc<dyn std::error::Error>, Some)))]
        source: Option<Rc<dyn std::error::Error>>,
    },
}
