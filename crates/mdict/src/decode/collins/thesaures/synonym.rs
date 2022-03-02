use std::fmt;

use itertools::Itertools;
use scraper::ElementRef;

use crate::{decode::collins::ToDisplay, static_selector};

#[derive(Debug)]
pub struct Synonym {
    pub word: String,
    pub sound: Option<String>,
    pub examples: Vec<String>,
    pub query_url: Option<String>,
}

impl<'a> From<ElementRef<'a>> for Synonym {
    fn from(elem: ElementRef<'a>) -> Self {
        let word = elem.select(static_selector!(".orth")).nth(0);
        let sound = elem.select(static_selector!(".sound")).nth(0);
        let examples = elem.select(static_selector!(".type-example"));
        let queryable = elem.select(static_selector!(".type-syn .ref")).nth(0);

        Synonym {
            word: word.unwrap().to_display(),
            sound: sound.and_then(|n| n.value().attr("data-src-mp3").map(|v| v.to_string())),
            examples: examples.map(|n| n.to_display()).collect_vec(),
            query_url: queryable.and_then(|n| n.value().attr("href").map(|v| v.to_string())),
        }
    }
}

impl fmt::Display for Synonym {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(query_url) = &self.query_url {
            write!(f, "[[{}][{}]]", query_url, self.word)?;
        } else {
            write!(f, "{}", self.word)?;
        }

        write!(f, " ")?;

        if let Some(sound) = &self.sound {
            write!(f, "[[{}][audio]]", sound)?;
        }

        write!(f, "\n")?;

        for example in &self.examples {
            write!(f, "- {}\n", example)?;
        }
        Ok(())
    }
}
