mod command;
mod util;
use clap;

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
            match command::jp::query(query).await {
                Ok(word) => println!("{}", &word.to_cli_str()),
                Err(e) => println!("{}", e),
            }
        }
    }
}
