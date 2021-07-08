mod command;
mod util;

use clap::Clap;
use command::{Opts, SubCommand};
use SubCommand::Dict;
use SubCommand::Translate;

use qt_core::{q_init_resource, qs};
use qt_widgets::QApplication;
use qt_qml::QQmlApplicationEngine;

#[tokio::main]
async fn main() {
    QApplication::init(|_| unsafe {
        q_init_resource!("resources");
        let _engine = QQmlApplicationEngine::from_q_string(&qs("qrc:/main.qml"));
        QApplication::exec()
    })

    // match Opts::parse().subcmd {
    //     Dict(dict) => dict.do_query().await,
    //     Translate(translate) => translate.do_query().await,
    // }
}
