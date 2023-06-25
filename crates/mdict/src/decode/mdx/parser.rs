use std::ops::RangeFrom;

use multimap::MultiMap;
use nom::{sequence::tuple, AsBytes, InputIter, InputLength, Slice};

use super::{
    common::{mdx_number, mdx_string, NomResult},
    content_block,
    dict_meta::{self, DictMeta},
    key_block::{self, KeyBlock},
    record_block::{self, RecordBlock},
    Dict, Result,
};

#[derive(Debug, Clone)]
pub struct KeyIndex {
    pub record_index: usize,
    pub block_pos: usize,
    pub key: String,
    pub block_size: usize,
}

impl KeyIndex {
    fn collect<I>(
        meta: &DictMeta,
        key_block: &KeyBlock<I>,
        record_block: &RecordBlock<I>,
    ) -> Result<MultiMap<String, KeyIndex>>
    where
        I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
    {
        let mut keys = MultiMap::new();

        let mut input = key_block.keys_input.clone();

        let mut prev_keyindex: Option<KeyIndex> = None;
        let mut prev_block_pos = 0;

        #[derive(Debug)]
        struct RecordBlockInfo {
            index: usize,
            begin_pos: usize,
            end_pos: usize,
        }

        let mut record_begin_pos = 0;

        let mut record_iter = record_block.infos.iter().enumerate().map(|(i, block)| {
            let result = RecordBlockInfo {
                index: i,
                begin_pos: record_begin_pos,
                end_pos: record_begin_pos + block.nb_decompressed,
            };

            record_begin_pos += block.nb_decompressed;

            result
        });

        let mut record = record_iter.next().unwrap();

        for item in key_block.infos.iter() {
            let (in_, data) =
                content_block::parse(input, item.nb_compressed, item.nb_decompressed)?;
            input = in_;

            let mut data_input = data.as_bytes();

            for _ in 0..item.n_entries {
                let (i_, (pos, key)) = tuple((mdx_number(meta), mdx_string(meta)))(data_input)?;
                data_input = i_;

                while record.end_pos <= pos {
                    record = record_iter.next().unwrap();
                }

                if let Some(mut prev) = prev_keyindex {
                    prev.block_size = pos - prev_block_pos;

                    keys.insert(prev.key.clone(), prev);
                }

                prev_keyindex = Some(KeyIndex {
                    record_index: record.index,
                    block_pos: pos - record.begin_pos,
                    key,
                    block_size: 0,
                });
                prev_block_pos = pos;
            }
        }

        if let Some(mut last) = prev_keyindex {
            let record_block = record_iter.last().unwrap_or(record);

            last.record_index = record_block.index;
            last.block_size = record_block.end_pos - prev_block_pos;

            keys.insert(last.key.clone(), last);
        }
        Ok(keys)
    }
}

fn parse_inner<I>(in_: I) -> NomResult<I, Dict<I>>
where
    I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let (in_, meta) = dict_meta::parse(in_)?;
    let (in_, key_block) = key_block::parse(in_, &meta)?;
    let (in_, record_block) = record_block::parse(in_, &meta)?;

    let n_record_block = record_block.infos.len();

    let keys = KeyIndex::collect(&meta, &key_block, &record_block).unwrap_or_default();

    Ok((
        in_,
        Dict {
            meta,
            key_block,
            record_block,
            keys,
            blocks: vec![Vec::new(); n_record_block],
        },
    ))
}

pub fn parse<I>(in_: I) -> Result<Dict<I>>
where
    I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let (_, dict) = parse_inner(in_)?;
    Ok(dict)
}
