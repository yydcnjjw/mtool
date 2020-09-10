mod hj_jp_dict;

use clap;
use std::io::Write;

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
    f(text_io::try_read!().unwrap());
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

    match hj_jp_dict::get_dict(&input).await {
        Ok(word_infos) => {
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
            print!("{}", hj_jp_dict::to_cli_str(word_infos.get(i).unwrap()));
        }
        Err(e) => match e {
            hj_jp_dict::Error::WordSuggestion(v) => {
                cli_read_choice_cb("word suggestions: ", &v, |i| {
                    println!("{}", v.get(i).unwrap());
                });
            }
            _ => panic!(e),
        },
    };
}
