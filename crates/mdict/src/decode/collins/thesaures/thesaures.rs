use scraper::Html;

use crate::static_selector;

use super::Sense;

#[derive(Debug)]
pub struct ThesauresResult {
    pub source: String,
    pub senses: Vec<Sense>,
}

fn source<'a>(doc: &'a Html) -> impl Iterator<Item = String> + 'a {
    doc.select(static_selector!(".headerThes > .entry_title"))
        .filter_map(|node| node.text().nth(0).map(|v| v.to_string()))
}

fn sense_list_iter<'a>(doc: &'a Html) -> anyhow::Result<impl Iterator<Item = Vec<Sense>> + 'a> {
    Ok(doc.select(static_selector!(".synonyms")).map(|node| {
        node.select(static_selector!(".sense"))
            .map(|e| Sense::from(e))
            .collect::<_>()
    }))
}

pub fn thesaures_list<'a>(doc: &'a Html) -> anyhow::Result<impl Iterator<Item = ThesauresResult> + 'a> {
    Ok(source(doc)
        .zip(sense_list_iter(doc)?)
        .map(|(source, senses)| ThesauresResult { source, senses }))
}
