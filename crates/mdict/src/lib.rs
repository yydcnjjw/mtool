pub mod common;
mod content_block;
mod key_block;
mod record_block;

pub mod dict_meta;
pub mod mdict;
pub mod mdsearch;

use std::{io, path::Path, result, string::FromUtf16Error};

use mdict::Mdict;
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
    NomInner(ErrorKind),
    #[error("Out of bouds {0} {1}")]
    OutOfBounds(usize, usize),
    #[error("{0}")]
    Nom(String),
}

impl<I> ParseError<I> for Error {
    fn from_error_kind(_input: I, kind: ErrorKind) -> Self {
        Error::NomInner(kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I, E> FromExternalError<I, E> for Error {
    fn from_external_error(_: I, kind: ErrorKind, _: E) -> Self {
        Error::NomInner(kind)
    }
}

type Result<T> = result::Result<T, Error>;
type NomResult<I, O, E = Error> = nom::IResult<I, O, E>;

pub fn parse(path: &Path) -> Result<Mdict> {
    let buf = common::read_file_to_buf(path);

    match mdict::parse(buf.as_slice()) {
        Ok((_, mdict)) => Ok(mdict),
        Err(e) => Err(Error::Nom(e.to_string())),
    }
}
