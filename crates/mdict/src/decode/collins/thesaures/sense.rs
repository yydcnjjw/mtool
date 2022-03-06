use itertools::Itertools;
use scraper::ElementRef;

use crate::{decode::collins::ToDisplay, static_selector};

use super::Synonym;

#[derive(Debug)]
pub struct Sense {
    pub pos: String,             // part of speech
    pub word: String,            // in the sense of `word`
    pub content: Option<String>, // definition content
    pub example: Option<String>, // definition example
    pub synonyms: Vec<Synonym>,
}

impl<'a> From<ElementRef<'a>> for Sense {
    fn from(elem: ElementRef<'a>) -> Self {
        let pos = elem.select(static_selector!(".headerSensePos")).nth(0);
        let word = elem.select(static_selector!(".headwordSense")).nth(0);
        let content = elem.select(static_selector!(".def")).nth(0);
        let example = elem.select(static_selector!(".type-example")).nth(0);

        Sense {
            pos: pos.unwrap().to_display(),
            word: word.unwrap().to_display(),
            content: content.map(|n| n.to_display()),
            example: example.map(|n| n.to_display()),
            synonyms: synonym_list(elem),
        }
    }
}

fn synonym_list<'a>(elem: ElementRef<'a>) -> Vec<Synonym> {
    elem.select(static_selector!(".blockSyn > .type-syn"))
        .map(|e| Synonym::from(e))
        .collect_vec()
}
