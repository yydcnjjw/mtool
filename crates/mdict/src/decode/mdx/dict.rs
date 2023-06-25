use std::ops::RangeFrom;

use anyhow::Context;
use itertools::Itertools;
use multimap::MultiMap;
use nom::multi::count;
use nom::number::complete::le_u8;
use nom::{InputIter, InputLength, Slice};

use super::parser::KeyIndex;
use super::{common::Resource, Result};
use super::{content_block, Error};
use super::{dict_meta::DictMeta, key_block::KeyBlock, record_block::RecordBlock};

#[derive(Debug)]
pub struct Dict<I> {
    pub meta: DictMeta,
    pub key_block: KeyBlock<I>,
    pub record_block: RecordBlock<I>,

    pub keys: MultiMap<String, KeyIndex>,
    pub blocks: Vec<Vec<u8>>,
}

impl<I> Dict<I>
where
    I: Slice<RangeFrom<usize>> + Clone + PartialEq + InputIter<Item = u8> + InputLength,
{
    fn try_get_block_or_decompress(&mut self, i: usize) -> Result<&[u8]> {
        let n_blocks = self.record_block.infos.len();
        let info = self
            .record_block
            .infos
            .get(i)
            .ok_or(Error::OutOfBounds(n_blocks, i))?;

        if self.blocks[i].len() == info.nb_decompressed {
            Ok(&self.blocks[i])
        } else {
            let offset = self.record_block.infos[0..i]
                .iter()
                .map(|info| info.nb_compressed)
                .sum();

            let (_, data) =
                count(le_u8, info.nb_compressed)(self.record_block.records_input.slice(offset..))?;

            let (_, decompressed_block) =
                content_block::parse(data.as_slice(), info.nb_compressed, info.nb_decompressed)?;

            {
                self.blocks[i] = decompressed_block;
            }

            Ok(&self.blocks[i])
        }
    }

    fn get_block(&self, i: usize) -> &[u8] {
        &self.blocks[i]
    }

    fn try_get_record_or_decompress(&mut self, key_index: &KeyIndex) -> Result<&[u8]> {
        let block = self.try_get_block_or_decompress(key_index.record_index)?;
        Ok(&block[key_index.block_pos..key_index.block_pos + key_index.block_size])
    }

    fn get_record(&self, key_index: &KeyIndex) -> &[u8] {
        let block = self.get_block(key_index.record_index);
        &block[key_index.block_pos..key_index.block_pos + key_index.block_size]
    }

    pub fn search<'a>(&'a mut self, text: &str) -> Result<(String, Resource<'a>)> {
        let text = text.trim();

        if text.is_empty() {
            return Err(Error::Other(anyhow::anyhow!("text is empty")));
        }

        let key_index = self
            .keys
            .get_vec(text)
            .context(format!("{} not found", text))?
            .iter()
            .find_or_first(|key_index| text == key_index.key)
            .cloned()
            .unwrap();
        let key = key_index.key.clone();

        self.try_get_record_or_decompress(&key_index)?;

        let record = self.get_record(&key_index);

        Ok((key, Resource::new(text, record, &self.meta)?))
    }
}
