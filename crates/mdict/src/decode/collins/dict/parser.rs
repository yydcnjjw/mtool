use itertools::Itertools;
use scraper::Html;

use super::{dict::dict_list, DictResult};

pub fn parse(doc: &str) -> anyhow::Result<Vec<DictResult>> {
    let doc = Html::parse_document(doc);
    let result = dict_list(&doc)?;
    Ok(result.collect_vec())
}
