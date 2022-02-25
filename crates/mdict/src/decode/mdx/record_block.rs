use std::ops::{IndexMut, RangeFrom};

use nom::{
    combinator::map, error::ParseError, multi::count, number::streaming::le_u8, sequence::tuple,
    IResult, InputIter, InputLength, Slice,
};

use super::{
    common::{mdx_number, NomResult},
    content_block,
    dict_meta::DictMeta,
    Error, Result,
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
pub struct RecordBlock {
    #[allow(dead_code)]
    header: RecordBlockHeader,
    pub infos: Vec<RecordBlockInfo>,
    blocks: Vec<Vec<u8>>,
}

impl RecordBlock {
    pub fn unzip_blocks(&mut self, indexs: &Vec<usize>) -> Result<()> {
        for i in indexs {
            self.unzip_block(*i)?;
        }

        Ok(())
    }

    fn unzip_block(&mut self, i: usize) -> Result<()> {
        let info = self
            .infos
            .get(i)
            .ok_or(Error::OutOfBounds(self.infos.len(), i))?;

        let block = self.blocks.get(i).unwrap();
        if block.len() == info.nb_decompressed {
            return Ok(());
        }

        let (_, decompressed_block) =
            content_block::parse(block.as_slice(), info.nb_compressed, info.nb_decompressed)?;

        let block = self.blocks.index_mut(i);
        block.clear();
        block.extend_from_slice(&decompressed_block);
        Ok(())
    }

    pub fn get_block<'a>(&'a self, i: usize) -> Result<&'a Vec<u8>> {
        Ok(self
            .blocks
            .get(i)
            .ok_or(Error::OutOfBounds(self.blocks.len(), i))?)
    }
}

pub fn parse<I>(in_: I, meta: &DictMeta) -> NomResult<I, RecordBlock>
where
    I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let (in_, header) = record_block_header(meta)(in_)?;
    let (in_, infos) = record_block_info(meta, &header)(in_)?;

    let mut input = in_.clone();

    let mut blocks = Vec::new();
    for info in infos.iter() {
        let input_ = input.clone();
        let (i_, block) = count(le_u8, info.nb_compressed)(input_)?;
        input = i_;
        blocks.push(block);
    }

    Ok((
        input,
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
