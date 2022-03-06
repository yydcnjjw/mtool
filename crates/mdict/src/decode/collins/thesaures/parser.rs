use scraper::Html;

use crate::static_selector;

use super::Sense;

#[derive(Debug)]
pub struct Thesaures {
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

fn thesaures_list<'a>(doc: &'a Html) -> anyhow::Result<impl Iterator<Item = Thesaures> + 'a> {
    Ok(source(doc)
        .zip(sense_list_iter(doc)?)
        .map(|(source, senses)| Thesaures { source, senses }))
}

pub fn parse(doc: &str) -> anyhow::Result<Vec<Thesaures>> {
    let doc = Html::parse_document(doc);
    Ok(thesaures_list(&doc).unwrap().collect::<Vec<_>>())
}

