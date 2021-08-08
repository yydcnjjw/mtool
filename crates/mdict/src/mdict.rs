use std::ops::{Index, RangeFrom};

use nom::{InputIter, InputLength, Slice};

use crate::{
    dict_meta::{self, DictMeta},
    key_block::{self, KeyBlock},
    mdsearch::MdSearchIndex,
    record_block::{self, RecordBlock},
    Error, NomResult, Result,
};

#[derive(Debug)]
pub struct Mdict {
    pub meta: DictMeta,
    pub key_block: KeyBlock,
    pub record_block: RecordBlock,
}

impl Mdict {
    pub fn search(&self, text: String) -> Vec<(String, MdSearchIndex)> {
        self.key_block
            .infos
            .iter()
            .enumerate()
            .flat_map(|(index, _)| {
                let keyindexs = self.key_block.blocks.index(index);

                keyindexs
                    .iter()
                    .enumerate()
                    .filter(|(_, keyindex)| keyindex.key.contains(&text))
                    .map(|(i, keyindex)| {
                        let mut pos = keyindex.pos;
                        let mut offset = 0;
                        let (block_index, block) = self
                            .record_block
                            .blocks
                            .iter()
                            .enumerate()
                            .find_map(|(i, block)| {
                                if pos < block.len() {
                                    Some((i, block))
                                } else {
                                    pos -= block.len();
                                    offset += block.len();
                                    None
                                }
                            })
                            .unwrap();

                        let next_pos = if i == keyindexs.len() - 1 {
                            block.len()
                        } else {
                            keyindexs.index(i + 1).pos - offset
                        };

                        (
                            keyindex.key.clone(),
                            MdSearchIndex {
                                block_index,
                                block_pos: pos,
                                data_size: next_pos - pos,
                            },
                        )
                    })
                    .collect::<Vec<(String, MdSearchIndex)>>()
            })
            .collect::<_>()
    }
}

pub fn parse<I>(in_: I) -> NomResult<I, Mdict>
where
    I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let (in_, meta) = dict_meta::parse(in_)?;
    let (in_, key_block) = key_block::parse(in_, &meta)?;
    let (in_, record_block) = record_block::parse(in_, &meta)?;

    Ok((
        in_,
        Mdict {
            meta,
            key_block,
            record_block,
        },
    ))
}

pub fn parse_result<I>(in_: I) -> Result<Mdict>
where
    I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    match parse(in_) {
        Ok((_, mdict)) => Ok(mdict),
        Err(e) => Err(e),
    }
}
