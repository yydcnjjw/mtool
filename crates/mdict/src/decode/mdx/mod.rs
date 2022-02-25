mod common;
mod content_block;
mod dict;
mod dict_meta;
mod error;
mod key_block;
mod parser;
mod record_block;

type Result<T> = std::result::Result<T, Error>;

pub use self::{
    common::Resource,
    dict::Dict,
    error::Error,
    parser::parse,
};
