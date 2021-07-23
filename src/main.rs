mod command;
mod util;

use clap::Clap;
use command::{Opts, SubCommand};
use SubCommand::Dict;
use SubCommand::Translate;
use SubCommand::Search;

// use qt_core::{q_init_resource, qs};
// use qt_qml::QQmlApplicationEngine;
// use qt_widgets::QApplication;

#[tokio::main]
async fn main() {
    // QApplication::init(|_| unsafe {
    //     q_init_resource!("resources");
    //     let _engine = QQmlApplicationEngine::from_q_string(&qs("qrc:/main.qml"));
    //     QApplication::exec()
    // })

    match Opts::parse().subcmd {
        Dict(dict) => dict.run().await,
        Translate(translate) => translate.run().await,
        Search(search) => search.run().await,
    }
}
