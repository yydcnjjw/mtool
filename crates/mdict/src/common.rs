use std::{fs::File, io::Read, ops::RangeFrom, path::Path};

use nom::{
    bytes::streaming::tag,
    combinator::map,
    error::ParseError,
    multi::many_till,
    number::streaming::{be_u32, be_u64, le_u16, le_u8},
    Compare, IResult, InputIter, InputLength, InputTake, Parser, Slice,
};

use crate::{Error, Result};

use super::dict_meta::DictMeta;

#[macro_export]
macro_rules! nom_return {
    ($in_:tt, $output_t:ty, $x:expr) => {
        match || -> crate::Result<$output_t> { Ok($x) }() {
            Ok(v) => Ok(($in_, v)),
            Err(e) => Err(nom::Err::Failure(e)),
        }
    };
}

pub fn cond_if<I, E, O, F1, F2>(
    cond: bool,
    mut f1: F1,
    mut f2: F2,
) -> impl FnMut(I) -> IResult<I, O, E>
where
    F1: Parser<I, O, E>,
    F2: Parser<I, O, E>,
{
    move |in_: I| {
        if cond {
            f1.parse(in_)
        } else {
            f2.parse(in_)
        }
    }
}

pub fn mdict_number<I, E>(meta: &DictMeta) -> impl FnMut(I) -> IResult<I, usize, E>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
    E: ParseError<I>,
{
    cond_if(
        meta.is_ver2(),
        map(be_u64, |v| v as usize),
        map(be_u32, |v| v as usize),
    )
}

const U8NULL: &'static [u8] = &[0u8];
const U16NULL: &'static [u8] = &[0u8, 0u8];

pub fn mdict_string<I, E>(meta: &DictMeta) -> impl FnMut(I) -> IResult<I, String, E>
where
    I: Clone
        + PartialEq
        + Slice<RangeFrom<usize>>
        + InputIter<Item = u8>
        + InputLength
        + InputTake
        + Compare<&'static [u8]>,
    E: ParseError<I>,
{
    cond_if(
        meta.encoding == "UTF-8",
        map(many_till(le_u8, tag(U8NULL)), |(v, _)| {
            String::from_utf8(v).unwrap_or_default()
        }),
        map(many_till(le_u16, tag(U16NULL)), |(v, _)| {
            String::from_utf16(&v).unwrap_or_default()
        }),
    )
}

#[derive(Debug)]
pub enum MdResource<'a> {
    Text(String),
    Raw(&'a [u8]),
}

impl<'a> MdResource<'a> {
    pub fn new(key: &String, data: &'a [u8], meta: &DictMeta) -> Result<MdResource<'a>> {
        if key.ends_with(".png") {
            Ok(MdResource::Raw(data))
        } else {
            match mdict_string::<_, Error>(meta)(data) {
                Ok((_, text)) => Ok(MdResource::Text(text)),
                Err(e) => Err(Error::Nom(e.to_string())),
            }
        }
    }
}
