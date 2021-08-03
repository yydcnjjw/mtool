mod common;
pub mod dict_meta;
pub mod mdx;

use std::{io, result, string::FromUtf16Error};

use nom::error::{ErrorKind, ParseError};
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

type Result<T> = result::Result<T, Error>;
type NomResult<I, O> = nom::IResult<I, O, Error>;
