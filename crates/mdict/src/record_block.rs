use std::ops::RangeFrom;

use nom::{
    combinator::map, error::ParseError, multi::count, sequence::tuple, IResult, InputIter,
    InputLength, Slice,
};

use crate::{common::mdict_number, content_block, dict_meta::DictMeta, NomResult};

#[derive(Debug)]
struct RecordBlockHeader {
    n_blocks: u64,
    n_entries: u64,
    nb_block_info: u64,
    nb_blocks: u64,
}

#[derive(Debug)]
struct RecordBlockInfo {
    nb_compressed: u64,
    nb_decompressed: u64,
}

#[derive(Debug)]
pub struct RecordBlock {
    header: RecordBlockHeader,
    infos: Vec<RecordBlockInfo>,
    pub blocks: Vec<Vec<u8>>,
}

pub fn parse<I>(in_: I, meta: &DictMeta) -> NomResult<I, RecordBlock>
where
    I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let (in_, header) = record_block_header(meta)(in_)?;
    let (mut in_, infos) = record_block_info(meta, &header)(in_)?;
    println!("{:?}", header);

    let blocks = infos
        .iter()
        .map(|item| {
            // TODO: unwrap
            let (i_, data) =
                content_block::parse(in_.clone(), item.nb_compressed, item.nb_decompressed)
                    .unwrap();
            in_ = i_;
            data
        })
        .collect::<_>();

    Ok((
        in_,
        RecordBlock {
            header,
            infos,
            blocks,
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
            mdict_number(meta),
            mdict_number(meta),
            mdict_number(meta),
            mdict_number(meta),
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
            tuple((mdict_number(meta), mdict_number(meta))),
            |(nb_compressed, nb_decompressed)| RecordBlockInfo {
                nb_compressed: nb_compressed - 8,
                nb_decompressed,
            },
        ),
        header.n_blocks as usize,
    )
}
