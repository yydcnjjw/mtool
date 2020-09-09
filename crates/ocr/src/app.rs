use std::process::Stdio;

use cxx::{CxxVector, UniquePtr};
use qt_core::{q_init_resource, qs, ApplicationAttribute, QCoreApplication};
use qt_gui::QGuiApplication;
use qt_qml::{QQmlApplicationEngine, QQmlImageProviderBase};
use tokio::{io::AsyncWriteExt, process::Command};

use crate::{
    config::Config,
    convert::{OCRConvert, TencentConvertor},
    Error, Result,
};

#[cxx::bridge(namespace = "rust")]
mod ffi {
    extern "Rust" {
        type App;
        fn ocr(&self, img: UniquePtr<CxxVector<u8>>) -> Result<()>;
    }

    unsafe extern "C++" {
        include!("ocr/screenshot/rust/screen_image_provider.hpp");
        include!("ocr/screenshot/rust/message.hpp");

        type ScreenImageProvider;
        fn new_screen_image_provider() -> *const ScreenImageProvider;

        fn qml_register_message(app: &App);
        fn qt_quit();
    }
}

async fn clip(text: &String) -> Result<()> {
    let mut child = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .stdin(Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().ok_or(Error::TakeStdio)?;
    stdin.write_all(text.as_bytes()).await?;
    Ok(())
}

async fn ocr_clip(config: Config, img: UniquePtr<CxxVector<u8>>) -> Result<()> {
    let result = TencentConvertor::new(config)
        .convert(img.as_slice())
        .await?;

    let text = result
        .iter()
        .fold(String::new(), |lhs, rhs| lhs + rhs + "\n");

    println!("{}", text);

    clip(&text).await?;
    Ok(())
}

pub struct App {
    config: Config,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn ocr(&self, img: UniquePtr<CxxVector<u8>>) -> Result<()> {
        let config = self.config.clone();
        tokio::spawn(async move {
            if let Err(e) = ocr_clip(config, img).await {
                println!("{}", e);
            }
            ffi::qt_quit();
        });

        Ok(())
    }

    pub fn run(&self) -> Result<()> {
        unsafe {
            QCoreApplication::set_attribute_1a(ApplicationAttribute::AAEnableHighDpiScaling);
        }

        QGuiApplication::init(|_| -> i32 {
            unsafe {
                q_init_resource!("screenshot");
                let engine = QQmlApplicationEngine::new();

                engine.add_image_provider(
                    &qs("screen"),
                    ffi::new_screen_image_provider() as *const QQmlImageProviderBase,
                );

                ffi::qml_register_message(self);

                engine.load_q_string(&qs("qrc:/main.qml"));

                QGuiApplication::exec()
            }
        })
    }
}
