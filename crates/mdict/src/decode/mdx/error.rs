use nom::error::{ErrorKind, FromExternalError, ParseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("nom parse error: {0:?}")]
    NomParse(ErrorKind),
    #[error("Out of bounds {0} {1}")]
    OutOfBounds(usize, usize),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl<I> ParseError<I> for Error {
    fn from_error_kind(_input: I, kind: ErrorKind) -> Self {
        Error::NomParse(kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I, E> FromExternalError<I, E> for Error {
    fn from_external_error(_: I, kind: ErrorKind, _: E) -> Self {
        Error::NomParse(kind)
    }
}

impl From<nom::Err<Error>> for Error {
    fn from(e: nom::Err<Error>) -> Self {
        match e {
            nom::Err::Incomplete(_) => Self::NomParse(ErrorKind::Eof),
            nom::Err::Error(e) => e,
            nom::Err::Failure(e) => e,
        }
    }
}
