use std::path::Path;

use crate::{
    common::{read_file_to_buf, MdResource},
    mdict::{self, Mdict},
    mdsearch::MdSearch,
    Result,
};

#[derive(Debug)]
pub struct Mdd {
    mdict: Mdict,
}

impl MdSearch for Mdd {
    fn search(&self, text: String) -> Vec<(String, MdResource)> {
        self.mdict
            .search(text)
            .iter()
            .map(|(key, index)| (key.clone(), MdResource::Raw(index.get(&self.mdict))))
            .collect::<_>()
    }
}

impl Mdd {
    pub fn parse(path: &Path) -> Result<Mdd> {
        let buf = read_file_to_buf(path);
        let mdict = mdict::parse_result(buf.as_slice())?;

        Ok(Mdd { mdict })
    }
}
