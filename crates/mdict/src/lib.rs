mod common;
pub mod dict_meta;
mod key_block;
pub mod mdx;
mod content_block;
mod record_block;
mod mdd;

use std::{io, result, string::FromUtf16Error};

use nom::error::{ErrorKind, FromExternalError, ParseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    De(#[from] quick_xml::DeError),
    #[error("{0}")]
    FromUtf16(#[from] FromUtf16Error),
    #[error("{0}")]
    IO(#[from] io::Error),
    #[error("{0}")]
    LZO(#[from] minilzo_rs::Error),
    #[error("NomError")]
    Nom(ErrorKind),
}

impl<I> ParseError<I> for Error {
    fn from_error_kind(_input: I, kind: ErrorKind) -> Self {
        Error::Nom(kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I, E> FromExternalError<I, E> for Error {
    fn from_external_error(_: I, kind: ErrorKind, _: E) -> Self {
        Error::Nom(kind)
    }
}

type Result<T> = result::Result<T, Error>;
type NomResult<I, O, E = Error> = nom::IResult<I, O, E>;
