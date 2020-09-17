#![feature(option_zip, iterator_fold_self)]

mod anki_api;
mod hj_jp_dict;

use crate::anki_api::AnkiNote;
use crate::anki_api::AnkiNoteOptions;
use clap;
use std::io::Write;
use std::str::FromStr;

fn cli_read_line<T>() -> Result<T, T::Err>
where
    T: FromStr,
{
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .expect("read line failed");
    buffer.trim().parse::<T>()
}

fn cli_read_choice(msg: &str, options: &Vec<String>) -> usize {
    println!("{}", msg);
    options
        .iter()
        .enumerate()
        .for_each(|(i, v)| println!("{}. {}", i, v));
    print!("Input Number: ");
    std::io::stdout().flush().unwrap();
    return cli_read_line().unwrap();
}

fn cli_read_choice_cb<F>(msg: &str, options: &Vec<String>, f: F)
where
    F: Fn(usize),
{
    println!("{}", msg);
    options
        .iter()
        .enumerate()
        .for_each(|(i, v)| println!("{}. {}", i, v));
    print!("Input Number: ");
    std::io::stdout().flush().unwrap();
    f(cli_read_line().unwrap());
}

fn cli_read_y_or_n(msg: &str) -> bool {
    print!("{}", msg);
    std::io::stdout().flush().unwrap();

    let c: String = cli_read_line().unwrap();

    match c.to_lowercase().as_ref() {
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => false,
    }
}

async fn get_dict(input: &str) {
    match hj_jp_dict::get_dict(input).await {
        Ok(mut word_infos) => {
            let mut i = 0;
            if word_infos.len() > 1 {
                i = cli_read_choice(
                    "multi words",
                    &word_infos
                        .iter()
                        .map(|v| format!("{}[{}]", v.expression, v.pronounce.pronounce))
                        .collect(),
                );
            }

            let word_info = word_infos.get_mut(i).unwrap();

            if word_info.expression == input || word_info.pronounce.pronounce == input {
                word_info.expression = format!(
                    "{}[{}]",
                    word_info.expression, word_info.pronounce.pronounce
                );
            }

            print!("{}", hj_jp_dict::to_cli_str(&word_info));

            if !cli_read_y_or_n("add note[Y/n]") {
                return;
            }

            let mut note = AnkiNote::new(&word_info, Option::None);
            if !anki_api::can_add_note(&note).await.unwrap() {
                println!("{}: duplicate!", input);
                return;
            }

            note.options = Some(AnkiNoteOptions {
                allow_duplicate: false,
                duplicate_scope: "deck".to_string(),
            });
            match anki_api::add_note(&note).await {
                Ok(_) => println!("success!"),
                Err(e) => println!("{}", e),
            }
        }
        Err(e) => match e {
            hj_jp_dict::Error::WordSuggestion(v) => {
                cli_read_choice_cb("word suggestions: ", &v, |i| {
                    println!("{}", v.get(i).unwrap());
                });
            }
            _ => panic!(e),
        },
    }
}

#[tokio::main]
async fn main() {
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
    get_dict(input).await;
}
