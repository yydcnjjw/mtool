use itertools::Itertools;
use scraper::Html;

use super::{thesaures::thesaures_list, ThesauresResult};

pub fn parse(doc: &str) -> anyhow::Result<Vec<ThesauresResult>> {
    let doc = Html::parse_document(doc);
    let result = thesaures_list(&doc)?;
    Ok(result.collect_vec())
}
