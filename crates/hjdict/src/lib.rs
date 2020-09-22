#![feature(option_zip, iterator_fold_self)]
use colored::Colorize;
use html5ever::rcdom::Handle;
use reqwest;
use soup::prelude::{NodeExt, QueryBuilderExt, Soup};
use std::fmt;

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) \
                          AppleWebKit/537.36 (KHTML, like Gecko) \
                          Chrome/69.0.3497.81 Safari/537.36";
const COOKIE: &str = "HJ_UID=0f406091-be97-6b64-f1fc-f7b2470883e9; \
                      HJ_CST=1; HJ_CSST_3=1; TRACKSITEMAP=3%2C; \
                      HJ_SID=393c85c7-abac-f408-6a32-a1f125d7e8c6; \
                      _REF=; HJ_SSID_3=4a460f19-c0ae-12a7-8e86-6e360f69ec9b; \
                      _SREF_3=; HJ_CMATCH=1";

const API_URL: &str = "https://www.hjdict.com/jp/jc/";

#[derive(Debug)]
pub struct JPWord {
    pub expression: String,
    pub pronounce: WordPronounce,
    pub simples: Vec<WordSimple>,
    pub details: Vec<WordDetail>,
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

#[derive(Debug)]
pub enum Error {
    NetRequest(reqwest::Error),
    NotFound(String),
    WordSuggestion(Vec<String>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            Error::NetRequest(e) => e.fmt(f),
            Error::NotFound(s) => write!(f, "HJ dict not found: {}", s),
            Error::WordSuggestion(v) => write!(f, "HJ dict word suggestion: {:?}", v),
        }
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Error {
        Error::NetRequest(e)
    }
}

type Result<T> = std::result::Result<T, Error>;

impl JPWord {
    fn new(doc: &Handle) -> Option<JPWord> {
        Some(JPWord {
            expression: word_expression(doc)?,
            pronounce: word_pronounce(doc)?,
            simples: word_simples(doc)?,
            details: word_details(doc)?,
        })
    }

    pub fn to_cli_str(&self) -> String {
        let mut result = String::new();
        {
            result += &format!("{}\n", self.expression);
        }
        {
            let pronounce = &self.pronounce;
            result += &format!(
                "[{}] [{}] {}\n",
                pronounce.pronounce, pronounce.kata, pronounce.tone
            );
        }
        {
            let simples = &self.simples;
            if simples.is_empty() {
                result += "No simple\n"
            } else {
                result += "Simple:\n";
                result += &simples.to_cli();
            }
        }
        {
            let details = &self.details;
            if !details.is_empty() {
                result += "Details:\n"
            }

            result += &details.to_cli();
        }
        return result.bold().to_string();
    }
}

pub trait OutputFormat {
    fn to_cli(&self) -> String;
    fn to_html(&self) -> String;
}

pub trait SimpleFormat {
    fn fmt_simple(&self) -> String;
}

pub trait DetailFormat {
    fn fmt_detail(&self) -> String;
}

impl SimpleFormat for Vec<WordSimple> {
    fn fmt_simple(&self) -> String {
        self.to_html()
    }
}

impl SimpleFormat for Vec<WordDetail> {
    fn fmt_simple(&self) -> String {
        let mut result = String::new();
        result += "<dl>\n";
        self.iter().for_each(|v| {
            result += &format!("<dt>{}</dt>\n", v.mean_type);
            result += "<dd>\n";
            result += "<ul>\n";
            v.means
                .iter()
                .for_each(|v| result += &format!("<li><span>{}</span></li>\n", v.cn_mean));
            result += "</ul>\n";
            result += "</dd>\n";
        });
        result += "</dl>\n";
        return result;
    }
}

impl DetailFormat for Vec<WordDetail> {
    fn fmt_detail(&self) -> String {
        self.to_html()
    }
}

impl OutputFormat for WordSimple {
    fn to_cli(&self) -> String {
        let mut result = String::new();
        result += &format!("{}\n", self.mean_type);
        self.means
            .iter()
            .for_each(|v| result += &format!("  - {}\n", v.red()));
        return result;
    }
    fn to_html(&self) -> String {
        let mut result = String::new();
        result += &format!("<dt>{}</dt>\n", self.mean_type);
        result += "<dd>\n";
        result += "<ul>\n";
        self.means.iter().for_each(|v| {
            result += &format!("<li><span>{}</span></li>\n", v);
        });
        result += "</ul>\n";
        result += "</dd>\n";
        return result;
    }
}

impl OutputFormat for WordDetail {
    fn to_cli(&self) -> std::string::String {
        let mut result = String::new();
        result += &format!("{}\n", self.mean_type);
        self.means.iter().for_each(|v| {
            result += &format!("  - {}", v.cn_mean.red());
            result += &format!("    {}\n", v.jp_mean);
            v.sentence
                .iter()
                .for_each(|v| result += &format!("    - {}    {}\n", v.sentence_jp, v.sentence_cn))
        });
        return result;
    }
    fn to_html(&self) -> std::string::String {
        let mut result = String::new();
        result += &format!("<dt>{}</dt>\n", self.mean_type);
        result += "<dd>\n";
        result += "<ul>\n";
        self.means.iter().for_each(|v| {
            result += "<li>\n";
            result += &format!("<span>{}</span>/<span>{}</span>\n", v.cn_mean, v.jp_mean);
            result += "<ul>\n";
            v.sentence.iter().for_each(|v| {
                result += "<li>\n";
                result += &format!(
                    "{} \\ {} [sound:{}]\n",
                    v.sentence_jp, v.sentence_cn, v.sentence_audio
                );
                result += "</li>\n";
            });
            result += "</ul>\n";
            result += "</li>\n";
        });
        result += "</ul>\n";
        result += "</dd>\n";
        return result;
    }
}

impl<T> OutputFormat for Vec<T>
where
    T: OutputFormat,
{
    fn to_cli(&self) -> String {
        self.iter().fold(String::new(), |r, v| r + &v.to_cli())
    }
    fn to_html(&self) -> String {
        let mut result = String::new();
        result += "<dl>\n";
        self.iter().for_each(|v| {
            result += &v.to_html();
        });
        result += "</dl>\n";
        return result;
    }
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
fn get_all_word_info(doc: &Soup) -> Vec<JPWord> {
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

pub async fn get_jp_dict(input: &str) -> Result<Vec<JPWord>> {
    let resp = reqwest::Client::new()
        .get(&format!("{}{}", API_URL, input))
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::COOKIE, COOKIE)
        .send()
        .await?;

    let text = resp.text().await?;

    let doc = Soup::new(&text);

    if is_not_found_page(&doc) {
        return Err(Error::NotFound(input.to_string()));
    }

    if let Some(v) = word_suggestions(&doc) {
        return Err(Error::WordSuggestion(v));
    }

    Result::Ok(get_all_word_info(&doc))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
