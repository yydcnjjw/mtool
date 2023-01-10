use std::cell::{Ref, RefCell, RefMut};
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
    pub blocks: RefCell<Vec<Vec<u8>>>,
}

impl<I> Dict<I>
where
    I: Slice<RangeFrom<usize>> + Clone + PartialEq + InputIter<Item = u8> + InputLength,
{
    fn get_block<'a>(&'a self, i: usize) -> Result<Ref<'a, Vec<u8>>> {
        let n_blocks = self.record_block.infos.len();
        let info = self
            .record_block
            .infos
            .get(i)
            .ok_or(Error::OutOfBounds(n_blocks, i))?;

        let offset = self.record_block.infos[0..i]
            .iter()
            .map(|info| info.nb_compressed)
            .sum();

        {
            let block = Ref::map(self.blocks.borrow(), |blocks| &blocks[i]);
            if block.len() == info.nb_decompressed {
                return Ok(block);
            }
        }

        let (_, data) =
            count(le_u8, info.nb_compressed)(self.record_block.records_input.slice(offset..))?;

        let (_, decompressed_block) =
            content_block::parse(data.as_slice(), info.nb_compressed, info.nb_decompressed)?;

        {
            let mut block = RefMut::map(self.blocks.borrow_mut(), |blocks| &mut blocks[i]);
            *block = decompressed_block;
        }

        Ok(Ref::map(self.blocks.borrow(), |blocks| &blocks[i]))
    }

    fn get_record<'a>(&'a self, key_index: &KeyIndex) -> Result<Ref<'a, [u8]>> {
        Ok(Ref::map(self.get_block(key_index.record_index)?, |block| {
            &block[key_index.block_pos..key_index.block_pos + key_index.block_size]
        }))
    }

    pub fn search<'a>(&'a self, text: &str) -> Result<(String, Resource<'a>)> {
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
            .unwrap();
        let key = key_index.key.clone();

        let record = self.get_record(&key_index)?;

        Ok((key, Resource::new(text, record, &self.meta)?))
    }
}
