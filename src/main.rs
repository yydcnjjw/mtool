#![feature(option_zip, iterator_fold_self)]
use clap;
use colored::*;
use html5ever::rcdom::Handle;
use reqwest::Result;
use soup::prelude::*;
use std::io::Write;

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
struct WordInfo {
    expression: String,
    pronounce: WordPronounce,
    simples: Vec<WordSimple>,
    details: Vec<WordDetail>,
}

#[derive(Default, Debug)]
struct WordPronounce {
    pronounce: String,
    kata: String,
    tone: String,
    audio: String,
}

#[derive(Default, Debug)]
struct WordSimple {
    mean_type: String,
    means: Vec<String>,
}

#[derive(Default, Debug)]
struct WordDetailSentence {
    sentence_jp: String,
    sentence_cn: String,
    sentence_audio: String,
}

#[derive(Default, Debug)]
struct WordDetailMean {
    jp_mean: String,
    cn_mean: String,
    sentence: Vec<WordDetailSentence>,
}

#[derive(Default, Debug)]
struct WordDetail {
    mean_type: String,
    means: Vec<WordDetailMean>,
}

#[derive(Debug, Clone)]
struct WordParseError;

impl WordInfo {
    fn new(doc: &Handle) -> Option<WordInfo> {
        Some(WordInfo {
            expression: word_expression(doc)?,
            pronounce: word_pronounce(doc)?,
            simples: word_simples(doc)?,
            details: word_details(doc)?,
        })
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

// match word pronounce:
// header.word-details-pane-header >  div.word-info > div.pronounces > span
fn word_pronounce(doc: &Handle) -> Option<WordPronounce> {
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
                        pronounce.pronounce = v.text().trim().to_string();
                    } else if let Some(class) = v.get("class") {
                        if class.contains("pronounce-value-jp") {
                            pronounce.tone = v.text().trim().to_string();
                        } else if class.contains("word-audio") {
                            pronounce.audio = if let Some(audio) = v.get("data-src") {
                                if audio.starts_with("https://") {
                                    audio.replacen("https://", "http://", 1)
                                } else {
                                    audio
                                }
                            } else {
                                String::new()
                            };
                        }
                    } else {
                        pronounce.kata = v.text().trim().to_string();
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
                                    if audio.starts_with("https://") {
                                        audio.replacen("https://", "http://", 1)
                                    } else {
                                        audio
                                    }
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
fn get_all_word_info(doc: &Soup) -> Vec<WordInfo> {
    doc.tag("section")
        .class("word-details-content")
        .find()
        .unwrap()
        .tag("div")
        .class("word-details-pane")
        .find_all()
        .filter_map(|v| WordInfo::new(&v))
        .collect()
}

fn to_cli(word_info: &WordInfo) -> String {
    let mut result = String::new();
    {
        result += &format!("{}\n", word_info.expression);
    }
    {
        let pronounce = &word_info.pronounce;
        result += &format!(
            "{} {} {}\n",
            pronounce.pronounce, pronounce.kata, pronounce.tone
        );
    }
    {
        let simples = &word_info.simples;
        if simples.is_empty() {
            result += "No simple\n"
        } else {
            result += "Simple:\n";
            simples.iter().for_each(|v| {
                result += &format!("{}\n", v.mean_type);
                v.means
                    .iter()
                    .for_each(|v| result += &format!("  - {}\n", v.red()));
            })
        }
    }
    {
        let details = &word_info.details;
        details.iter().for_each(|v| {
            result += &format!("{}\n", v.mean_type);
            v.means.iter().for_each(|v| {
                result += &format!("  - {}", v.cn_mean.red());
                result += &format!("    {}\n", v.jp_mean);
                v.sentence.iter().for_each(|v| {
                    result += &format!("    - {}    {}\n", v.sentence_jp, v.sentence_cn)
                })
            })
        })
    }
    return result.bold().to_string();
}

fn cli_read_choice(msg: &str, options: &Vec<String>) -> usize {
    println!("{}", msg);
    options
        .iter()
        .enumerate()
        .for_each(|(i, v)| println!("{}. {}", i, v));
    print!("Input Number: ");
    std::io::stdout().flush().unwrap();
    return text_io::try_read!().unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = clap::App::new("my tool")
        .version("0.1.0")
        .author("yydcnjjw <yydcnjjw@gmail.com>")
        .about("my tool")
        .arg(
            clap::Arg::with_name("input")
                .help("input jp")
                .required(true)
                .index(1),
        )
        .get_matches();

    let input = matches.value_of("input").unwrap();

    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}{}", API_URL, input))
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::COOKIE, COOKIE)
        .send()
        .await?;
    let text = resp.text().await?;

    let doc = Soup::new(&text);

    if is_not_found_page(&doc) {
        println!("Not found {}", input);
        return Result::Ok(());
    }

    if let Some(v) = word_suggestions(&doc) {
        cli_read_choice("word suggestions: ", &v);
        return Result::Ok(());
    }

    let mut word_choice = 0;
    let mut is_multi_word = false;
    if let Some(v) = multi_word(&doc) {
        word_choice = cli_read_choice(
            "multi word: ",
            &v.iter().map(|(v1, v2)| format!("{}{}", v1, v2)).collect(),
        );
        is_multi_word = true;
    }

    if let Some(v) = get_all_word_info(&doc).get(word_choice) {
        print!("{}", to_cli(&v));
    }

    Result::Ok(())
}
