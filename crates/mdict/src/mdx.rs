use std::path::Path;

use crate::{
    common::{mdict_string, read_file_to_buf, MdResource},
    mdict::{self, Mdict},
    mdsearch::MdSearch,
    NomResult, Result,
};

#[derive(Debug)]
pub struct Mdx {
    mdict: Mdict,
}

impl MdSearch for Mdx {
    fn search(&self, text: String) -> Vec<(String, MdResource)> {
        self.mdict
            .search(text)
            .iter()
            .map(|(key, index)| {
                let r: NomResult<&[u8], String> =
                    mdict_string(&self.mdict.meta)(index.get(&self.mdict));
                (key.clone(), MdResource::Text(r.unwrap().1))
            })
            .collect::<_>()
    }
}

impl Mdx {
    pub fn parse(path: &Path) -> Result<Mdx> {
        let buf = read_file_to_buf(path);
        let mdict = mdict::parse_result(buf.as_slice())?;

        Ok(Mdx { mdict })
    }
}
