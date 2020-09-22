#![feature(option_zip, iterator_fold_self)]

use crate::anki_api::AnkiNote;
use crate::anki_api::AnkiNoteOptions;
use clap;

async fn save_dict(word_info: &hj_jp_dict::WordInfo) -> Result<(), Box<dyn std::error::Error>> {
    if !cli_op::read_y_or_n("add note[Y/n]") {
        return Ok(());
    }

    let mut note = AnkiNote::new(&word_info, Option::None);
    if !anki_api::can_add_note(&note).await? {
        println!("{}: duplicate!", word_info.expression);
        return Ok(());
    }
    note.options = Some(AnkiNoteOptions {
        allow_duplicate: false,
        duplicate_scope: "deck".to_string(),
    });
    anki_api::add_note(&note).await?;
    Ok(println!("success!"))
}

#[tokio::main]
async fn main() {
    let matches = clap::App::new("my tool")
        .version("0.1.0")
        .author("yydcnjjw <yydcnjjw@gmail.com>")
        .about("my tool")
        .subcommand(
            clap::SubCommand::with_name("jp")
                .about("jp dict query")
                .arg(
                    clap::Arg::with_name("query")
                        .help("jp word for query")
                        .required(true)
                        .index(1),
                )
                .arg(
                    clap::Arg::with_name("save")
                        .short("s")
                        .help("jp word for save"),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("jp") {
        if let Some(query) = matches.value_of("query") {
            match hj_jp_dict::query_dict(query).await {
                Ok(word) => println!("{}", hj_jp_dict::to_cli_str(&word)),
                Err(e) => println!("{}", e),
            }
        }
    }
}
