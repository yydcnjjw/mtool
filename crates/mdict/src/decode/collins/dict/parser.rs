use itertools::Itertools;
use scraper::Html;

use super::{dict::dict_list, DictResult};

pub fn parse(doc: &str) -> Result<Vec<DictResult>, anyhow::Error> {
    let doc = Html::parse_document(doc);
    let result = dict_list(&doc)?;
    Ok(result.collect_vec())
}
