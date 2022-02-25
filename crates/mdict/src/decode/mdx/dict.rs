use super::{common::Resource, key_block::KeyIndex, Result};
use super::{dict_meta::DictMeta, key_block::KeyBlock, record_block::RecordBlock};

#[derive(Debug)]
pub struct Dict {
    pub meta: DictMeta,
    pub key_block: KeyBlock,
    pub record_block: RecordBlock,
}

#[derive(Debug)]
struct RecordIndex {
    pub block_index: usize,
    pub block_pos: usize,
    pub data_size: usize,
}

impl Dict {
    fn get_record<'a>(&'a self, index: &RecordIndex) -> Result<&'a [u8]> {
        Ok(&self.record_block.get_block(index.block_index)?
            [index.block_pos..index.block_pos + index.data_size])
    }

    pub fn search<'a>(&'a mut self, text: &String) -> Vec<(String, Resource<'a>)> {
        let search_indexs = self.search_index(|key| key == text);

        if let Err(_) = self.record_block.unzip_blocks(
            &search_indexs
                .iter()
                .map(|(_, index)| index.block_index)
                .collect::<_>(),
        ) {
            return Vec::new();
        }

        let mut krv = Vec::new();
        for (key, index) in search_indexs.iter() {
            let kr = match self.get_record(index) {
                Ok(data) => match Resource::new(text, data, &self.meta) {
                    Ok(resource) => Some((key.clone(), resource)),
                    Err(_) => None,
                },
                Err(_) => None,
            };
            if let Some(v) = kr {
                krv.push(v);
            }
        }
        krv
    }

    fn get_record_index(
        &self,
        keyindex: &KeyIndex,
        keyindex_pos: usize,
        key_block_pos: usize,
    ) -> RecordIndex {
        let keyindexs = self.key_block.blocks.get(key_block_pos).unwrap();
        let mut pos = keyindex.pos;
        let mut offset = 0;
        let (block_index, block_size) = self
            .record_block
            .infos
            .iter()
            .enumerate()
            .find_map(|(i, info)| {
                let decompressed_size = info.nb_decompressed;
                if pos < decompressed_size {
                    Some((i, decompressed_size))
                } else {
                    pos -= decompressed_size;
                    offset += decompressed_size;
                    None
                }
            })
            .unwrap();

        let next_pos = if keyindex_pos == keyindexs.len() - 1 {
            block_size
        } else {
            keyindexs[keyindex_pos + 1].pos - offset
        };

        RecordIndex {
            block_index,
            block_pos: pos,
            data_size: next_pos - pos,
        }
    }

    fn search_index<F>(&self, pred: F) -> Vec<(String, RecordIndex)>
    where
        F: Fn(&String) -> bool,
    {
        self.key_block
            .infos
            .iter()
            .enumerate()
            .flat_map(|(index, _)| {
                self.key_block
                    .blocks
                    .get(index)
                    .unwrap()
                    .iter()
                    .enumerate()
                    .filter(|(_, keyindex)| pred(&keyindex.key))
                    .map(|(i, keyindex)| {
                        (
                            keyindex.key.clone(),
                            self.get_record_index(keyindex, i, index),
                        )
                    })
                    .collect::<Vec<(String, RecordIndex)>>()
            })
            .collect::<_>()
    }
}
