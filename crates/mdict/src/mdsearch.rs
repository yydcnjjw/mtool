use std::ops::Index;

use crate::{common::MdResource, mdict::Mdict};

pub trait MdSearch {
    fn search(&self, text: String) -> Vec<(String, MdResource)>;
}

#[derive(Debug)]
pub struct MdSearchIndex {
    pub block_index: usize,
    pub block_pos: usize,
    pub data_size: usize,
}

impl MdSearchIndex {
    pub fn get<'a>(&self, mdict: &'a Mdict) -> &'a [u8] {
        println!("{:?}", self);
        &mdict.record_block.blocks.index(self.block_index)
            [self.block_pos..self.block_pos + self.data_size]
    }
}
