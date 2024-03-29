use soup::prelude::*;
use html5ever::rcdom::Handle;

use super::{Error, Result};

#[derive(Debug)]
pub struct JPWord {
    pub expression: String,
    pub pronounce: WordPronounce,
    pub simples: Vec<WordSimple>,
    pub details: Vec<WordDetail>,
}

impl JPWord {
    fn new(doc: &Handle) -> Option<JPWord> {
        Some(JPWord {
            expression: word_expression(doc)?,
            pronounce: word_pronounce(doc)?,
            simples: word_simples(doc)?,
            details: word_details(doc)?,
        })
    }
}

#[derive(Default, Debug)]
pub struct WordPronounce {
    pub pronounce: String,
    pub kata: String,
    pub tone: String,
    pub audio: String,
}

#[derive(Default, Debug)]
pub struct WordSimple {
    pub mean_type: String,
    pub means: Vec<String>,
}

#[derive(Default, Debug)]
pub struct WordDetailSentence {
    pub sentence_jp: String,
    pub sentence_cn: String,
    pub sentence_audio: String,
}

#[derive(Default, Debug)]
pub struct WordDetailMean {
    pub jp_mean: String,
    pub cn_mean: String,
    pub sentence: Vec<WordDetailSentence>,
}

#[derive(Default, Debug)]
pub struct WordDetail {
    pub mean_type: String,
    pub means: Vec<WordDetailMean>,
}

// not found page: div.word-notfound-inner
fn is_not_found_page(doc: &Soup) -> bool {
    doc.tag("div").class("word-notfound-inner").find().is_some()
}

// word suggestions page: div.word-suggestions > ul > li > a
fn word_suggestions(doc: &Soup) -> Option<Vec<String>> {
    doc.tag("div")
        .class("word-suggestions")
        .find()?
        .tag("ul")
        .find()
        .map(|v| {
            v.tag("li")
                .find_all()
                .filter_map(|v| v.tag("a").find().map(|v| v.text().trim().to_string()))
                .collect()
        })
}

// match multi word: header.word-details-header > ul > li
#[allow(dead_code)]
fn multi_word(doc: &Soup) -> Option<Vec<(String, String)>> {
    doc.tag("header")
        .class("word-details-header")
        .find()?
        .tag("ul")
        .find()
        .map(|v| {
            v.tag("li")
                .find_all()
                .filter_map(|v| {
                    v.tag("h2")
                        .find()
                        .map(|v| v.text().trim().to_string())
                        .zip(v.tag("span").find().map(|v| v.text().trim().to_string()))
                })
                .collect()
        })
}

// match word expression:
// header.word-details-pane-header > div.word-info > div.word-text > h2
fn word_expression(doc: &Handle) -> Option<String> {
    doc.tag("header")
        .class("word-details-pane-header")
        .find()?
        .tag("div")
        .class("word-info")
        .find()?
        .tag("div")
        .class("word-text")
        .find()?
        .tag("h2")
        .find()
        .map(|v| v.text().trim().to_string())
}

fn replace_https_to_http(s: &String) -> String {
    if s.starts_with("https://") {
        s.replacen("https://", "http://", 1)
    } else {
        s.to_string()
    }
}

// match word pronounce:
// header.word-details-pane-header >  div.word-info > div.pronounces > span
fn word_pronounce(doc: &Handle) -> Option<WordPronounce> {
    let remove_square_brackets = |s: &str| {
        regex::Regex::new(r"\[(.*)\]")
            .unwrap()
            .captures_iter(s)
            .fold(String::new(), |v1, v2| v1 + &v2[1])
    };

    doc.tag("header")
        .class("word-details-pane-header")
        .find()?
        .tag("div")
        .class("word-info")
        .find()?
        .tag("div")
        .class("pronounces")
        .find()
        .map(|v| {
            v.tag("span").find_all().enumerate().fold(
                WordPronounce::default(),
                |mut pronounce, (i, v)| {
                    if i == 0 {
                        pronounce.pronounce = remove_square_brackets(v.text().trim());
                    } else if let Some(class) = v.get("class") {
                        if class.contains("pronounce-value-jp") {
                            pronounce.tone = v.text().trim().to_string();
                        } else if class.contains("word-audio") {
                            pronounce.audio = if let Some(audio) = v.get("data-src") {
                                replace_https_to_http(&audio)
                            } else {
                                String::new()
                            };
                        }
                    } else {
                        pronounce.kata = remove_square_brackets(v.text().trim());
                    }
                    return pronounce;
                },
            )
        })
}

// match word simple
// header.word-details-pane-header > div.simple
fn word_simples(doc: &Handle) -> Option<Vec<WordSimple>> {
    doc.tag("header")
        .class("word-details-pane-header")
        .find()?
        .tag("div")
        .class("simple")
        .find()
        .map(|v| {
            v.tag("ul")
                .find_all()
                .zip(
                    v.tag("h2")
                        .find_all()
                        .map(|v| v.text().trim().to_string())
                        .chain(std::iter::repeat(String::new())),
                )
                .map(|(details, word_type)| WordSimple {
                    mean_type: word_type,
                    means: details
                        .tag("li")
                        .find_all()
                        .map(|v| v.text().trim().to_string())
                        .map(|v| {
                            regex::Regex::new(r"\d+\.")
                                .unwrap()
                                .replacen(&v, 1, "")
                                .to_string()
                        })
                        .collect(),
                })
                .collect()
        })
}

fn word_detail_sentence(doc: &Handle) -> Vec<WordDetailSentence> {
    doc.tag("li")
        .find_all()
        .map(|v| {
            let mut sentence: WordDetailSentence = Default::default();
            v.tag("p").find_all().enumerate().for_each(|(i, v)| {
                match i {
                    0 => {
                        sentence.sentence_jp = v.text().trim().to_string();
                        sentence.sentence_audio = v
                            .tag("span")
                            .class("word-audio")
                            .find()
                            .map(|v| {
                                if let Some(audio) = v.get("data-src") {
                                    replace_https_to_http(&audio)
                                } else {
                                    String::new()
                                }
                            })
                            .unwrap_or(String::new());
                    }
                    1 => {
                        sentence.sentence_cn = v.text().trim().to_string();
                    }
                    _ => (),
                };
            });
            return sentence;
        })
        .collect()
}

fn word_detail_mean(doc: &Handle) -> Vec<WordDetailMean> {
    doc.tag("dd")
        .find_all()
        .filter_map(|v| {
            let mut desc_mean: WordDetailMean = Default::default();
            v.tag("h3")
                .find()?
                .tag("p")
                .find_all()
                .enumerate()
                .for_each(|(i, v)| match i {
                    0 => desc_mean.jp_mean = v.text().trim().to_string(),
                    1 => desc_mean.cn_mean = v.text().trim().to_string(),
                    _ => (),
                });
            desc_mean.sentence = word_detail_sentence(&v);
            return Some(desc_mean);
        })
        .collect()
}

// match word detail
// div.word-details-item-content > section.detail-groups > dl
fn word_details(doc: &Handle) -> Option<Vec<WordDetail>> {
    doc.tag("div")
        .class("word-details-item-content")
        .find()?
        .tag("section")
        .class("detail-groups")
        .find()
        .map(|v| {
            v.tag("dl")
                .find_all()
                .map(|v| WordDetail {
                    mean_type: v
                        .tag("dt")
                        .find()
                        .map(|v| v.text().trim().to_string())
                        .unwrap_or(String::new()),
                    means: word_detail_mean(&v),
                })
                .collect()
        })
}

// match all word info:
// section.word-details-content > div.word-details-pane
fn collect_word_info(doc: &Soup) -> Vec<JPWord> {
    doc.tag("section")
        .class("word-details-content")
        .find()
        .unwrap()
        .tag("div")
        .class("word-details-pane")
        .find_all()
        .filter_map(|v| JPWord::new(&v))
        .collect()
}

pub fn parse(query: &String, text: &String) -> Result<Vec<JPWord>> {
    let doc = Soup::new(&text);

    if is_not_found_page(&doc) {
        return Err(Error::NotFound(query.to_string()));
    }

    if let Some(v) = word_suggestions(&doc) {
        return Err(Error::WordSuggestion(v));
    }

    Ok(collect_word_info(&doc))
}
