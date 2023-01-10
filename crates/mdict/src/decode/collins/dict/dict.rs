use itertools::Itertools;
use regex::Regex;
use scraper::{ElementRef, Html};
use selectors::{attr::CaseSensitivity, Element};

use crate::{decode::ToDisplay, static_selector};

fn normalize_text(text: &str) -> String {
    let text = text.replace("\n", " ");
    Regex::new(r#" +"#)
        .unwrap()
        .replace_all(&text, " ")
        .to_string()
}

#[derive(Debug)]
pub struct WordForm {
    pub form: String,
    pub content: Option<String>,
    pub sound: Option<String>,
}

impl<'a> From<ElementRef<'a>> for WordForm {
    fn from(elem: ElementRef<'a>) -> Self {
        let form = elem.to_display();
        let form = form.trim_start_matches(", ");
        let form = normalize_text(&form);

        let mut self_ = Self {
            form,
            content: None,
            sound: None,
        };

        for n in elem.next_siblings().take(2) {
            let elem = ElementRef::wrap(n).unwrap();

            if elem
                .value()
                .has_class("orth", CaseSensitivity::AsciiCaseInsensitive)
            {
                self_.content = Some(elem.to_display());
            } else if let Some(n) = elem.select(static_selector!(".sound")).nth(0) {
                self_.sound = n.value().attr("data-src-mp3").map(|n| n.to_string());
            } else {
                break;
            }
        }

        self_
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Text {
    pub content: String,
    pub query_url: Option<String>,
    pub bold: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Define {
    pub content: Vec<Text>,
}

impl<'a> From<ElementRef<'a>> for Define {
    fn from(elem: ElementRef<'a>) -> Self {
        let content = elem
            .children()
            .enumerate()
            .map(|(i, n)| {
                let mut text = if n.value().is_text() {
                    let content = n.value().as_text().unwrap();
                    Text {
                        content: normalize_text(&content),
                        query_url: None,
                        bold: false,
                    }
                } else {
                    let elem = ElementRef::wrap(n).unwrap();
                    let content = elem.to_display();
                    Text {
                        content: normalize_text(&content),
                        query_url: if elem
                            .value()
                            .has_class("type-def", CaseSensitivity::AsciiCaseInsensitive)
                        {
                            elem.value().attr("href").map(|n| n.to_string())
                        } else {
                            None
                        },
                        bold: elem
                            .value()
                            .has_class("rend-b", CaseSensitivity::AsciiCaseInsensitive),
                    }
                };
                if i == 0 {
                    text.content = text.content.trim_start().into();
                }
                text
            })
            .collect_vec();

        Self { content }
    }
}

#[derive(Debug, PartialEq)]
pub struct Example {
    pub content: String,
    pub syntax: Option<String>,
    pub sound: Option<String>,
}

impl<'a> From<ElementRef<'a>> for Example {
    fn from(elem: ElementRef<'a>) -> Self {
        let content = if let Some(n) = elem.select(static_selector!(".quote")).nth(0) {
            n
        } else if elem
            .value()
            .has_class("quote", CaseSensitivity::AsciiCaseInsensitive)
        {
            elem
        } else {
            unreachable!("{}", elem.to_display())
        };
        let syntax = elem.select(static_selector!(".type-syntax")).nth(0);
        let sound = elem.select(static_selector!(".sound")).nth(0);

        Self {
            content: normalize_text(&content.to_display()),
            syntax: syntax.map(|n| n.to_display()),
            sound: sound.and_then(|n| n.value().attr("data-src-mp3").map(|n| n.to_string())),
        }
    }
}

#[derive(Debug)]
pub struct XR {
    pub phrase: String,
    pub query_url: String,
}

impl<'a> From<ElementRef<'a>> for XR {
    fn from(elem: ElementRef<'a>) -> Self {
        Self {
            phrase: elem.to_display(),
            query_url: elem.value().attr("href").unwrap().to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Sense {
    pub define: Option<Define>,
    pub gram: Option<String>,
    pub examples: Vec<Example>,
    pub xr_hint: Option<String>,
    pub xrs: Vec<XR>,
    pub sub_sense: Vec<Sense>,
}

impl<'a> From<ElementRef<'a>> for Sense {
    fn from(elem: ElementRef<'a>) -> Self {
        let gram = elem
            .select(static_selector!(".subc"))
            .chain(elem.select(static_selector!(".type-register")))
            .chain(elem.select(static_selector!(".type-subj")))
            .chain(elem.select(static_selector!(".type-geo")))
            .nth(0);

        // .sense(1/2/3) > (.sensenum + .sense(a/b/c)) > .sensenum
        let sub_sense = elem
            .select(static_selector!(".sense"))
            .filter(|n| {
                n.select(static_selector!(".sensenum"))
                    .nth(0)
                    .map(|n| n.to_display().chars().any(|c| c.is_ascii_alphabetic()))
                    .is_some()
            })
            .map(|n| Sense::from(n))
            .collect_vec();

        let def = elem
            .select(static_selector!(".def"))
            .map(|n| Define::from(n))
            .filter(|def| {
                !sub_sense
                    .iter()
                    .filter_map(|s| s.define.as_ref())
                    .contains(def)
            })
            .nth(0);

        let examples = elem
            .select(static_selector!(".type-example"))
            .map(|n| Example::from(n))
            .filter(|e| !sub_sense.iter().flat_map(|s| s.examples.iter()).contains(e));

        let xr_hint = elem.select(static_selector!(".xr > .lbl")).nth(0);

        let xrs = elem
            .select(static_selector!(".xr > .ref"))
            .chain(elem.select(static_selector!("a.xr")));

        Self {
            gram: gram.map(|n| n.to_display()),
            define: def,
            examples: examples.collect_vec(),
            xr_hint: xr_hint.map(|n| n.to_display()),
            xrs: xrs.map(|n| XR::from(n)).collect_vec(),
            sub_sense,
        }
    }
}

#[derive(Debug)]
pub struct Hom {
    pub pos: Option<String>,
    pub syntax: Option<String>,
    pub senses: Vec<Sense>,
}

impl<'a> From<ElementRef<'a>> for Hom {
    fn from(elem: ElementRef<'a>) -> Self {
        let pos = elem.select(static_selector!(".pos")).nth(0);
        let syntax = elem
            .select(static_selector!(".gramGrp > .type-syntax"))
            .nth(0);

        // match
        // .hom > .sense > .sensenum
        // .hom > .sensenum
        let mut senses = elem
            .select(static_selector!(".sensenum"))
            .filter(|n| n.to_display().chars().any(|c| c.is_ascii_digit()))
            .map(|n| n.parent_element().unwrap())
            .map(|n| Sense::from(n))
            .collect_vec();

        // .hom > .sense > !.sensenum
        if senses.is_empty() {
            senses.extend(
                elem.select(static_selector!(".sense"))
                    .map(|n| Sense::from(n.parent_element().unwrap()))
                    .collect_vec(),
            );
        }

        Self {
            pos: pos.map(|n| n.to_display()),
            syntax: syntax.map(|n| n.to_display()),
            senses,
        }
    }
}

#[derive(Debug)]
pub struct DictResult {
    pub source: Option<String>,
    pub word: String,
    pub pronounce: Option<String>,
    pub sound: Option<String>,
    pub wfs: Vec<WordForm>,
    pub homs: Vec<Hom>,
}

impl<'a> From<ElementRef<'a>> for DictResult {
    fn from(elem: ElementRef<'a>) -> Self {
        let source = elem.select(static_selector!(".dictname")).nth(0);
        let word = elem
            .select(static_selector!(".title_container .orth"))
            .nth(0)
            .unwrap();

        let pronounce = elem.select(static_selector!(".mini_h2 .pron")).nth(0);
        let sound = elem
            .select(static_selector!(".mini_h2 .pron .sound"))
            .nth(0);

        let wfs = elem.select(static_selector!(".definitions > .form > .type-gram"));

        let homs = elem.select(static_selector!(".definitions > .hom"));

        Self {
            source: source.map(|n| n.to_display()),
            word: word.to_display(),
            pronounce: pronounce.map(|n| n.to_display()),
            sound: sound.and_then(|n| n.value().attr("data-src-mp3").map(|v| v.to_string())),
            wfs: wfs.map(|n| WordForm::from(n)).collect_vec(),
            homs: homs.map(|n| Hom::from(n)).collect_vec(),
        }
    }
}

pub fn dict_list<'a>(
    doc: &'a Html,
) -> Result<impl Iterator<Item = DictResult> + 'a, anyhow::Error> {
    Ok(doc
        .select(static_selector!(".dictlink"))
        .map(|n| DictResult::from(n)))
}
