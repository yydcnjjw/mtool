use std::ops::RangeFrom;

use nom::{
    combinator::map, error::ParseError, multi::count, sequence::tuple, IResult, InputIter,
    InputLength, Slice,
};

use super::{
    common::{mdx_number, NomResult},
    dict_meta::DictMeta,
};

#[derive(Debug)]
struct RecordBlockHeader {
    n_blocks: usize,
    #[allow(dead_code)]
    n_entries: usize,
    #[allow(dead_code)]
    nb_block_info: usize,
    #[allow(dead_code)]
    nb_blocks: usize,
}

#[derive(Debug)]
pub struct RecordBlockInfo {
    pub nb_compressed: usize,
    pub nb_decompressed: usize,
}

#[derive(Debug)]
pub struct RecordBlock<T> {
    #[allow(dead_code)]
    header: RecordBlockHeader,
    pub infos: Vec<RecordBlockInfo>,
    pub records_input: T,
}

pub fn parse<I>(in_: I, meta: &DictMeta) -> NomResult<I, RecordBlock<I>>
where
    I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let (in_, header) = record_block_header(meta)(in_)?;
    let (in_, infos) = record_block_info(meta, &header)(in_)?;

    Ok((
        in_.clone(),
        RecordBlock {
            header,
            infos,
            records_input: in_.clone(),
        },
    ))
}

fn record_block_header<I, E>(meta: &DictMeta) -> impl FnMut(I) -> IResult<I, RecordBlockHeader, E>
where
    I: Clone + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
    E: ParseError<I>,
{
    map(
        tuple((
            mdx_number(meta),
            mdx_number(meta),
            mdx_number(meta),
            mdx_number(meta),
        )),
        |(n_blocks, n_entries, nb_block_info, nb_blocks)| RecordBlockHeader {
            n_entries,
            n_blocks,
            nb_block_info,
            nb_blocks,
        },
    )
}
fn record_block_info<I, E>(
    meta: &DictMeta,
    header: &RecordBlockHeader,
) -> impl FnMut(I) -> IResult<I, Vec<RecordBlockInfo>, E>
where
    I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
    E: ParseError<I>,
{
    count(
        map(
            tuple((mdx_number(meta), mdx_number(meta))),
            |(nb_compressed, nb_decompressed)| RecordBlockInfo {
                nb_compressed,
                nb_decompressed,
            },
        ),
        header.n_blocks,
    )
}
