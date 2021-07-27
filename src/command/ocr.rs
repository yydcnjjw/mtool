#![windows_subsystem = "windows"]

use clap::Clap;

use qt_core::{q_init_resource, qs};
use qt_gui::QGuiApplication;
use qt_qml::QQmlApplicationEngine;

#[derive(Clap)]
pub struct Ocr {}

impl Ocr {
    pub async fn run(&self) {
        QGuiApplication::init(|_| unsafe {
            q_init_resource!("screenshot");
            let _engine = QQmlApplicationEngine::from_q_string(&qs("qrc:/screenshot.qml"));
            QGuiApplication::exec()
        })
    }
}
