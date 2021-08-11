pub mod config;

use std::{
    io::Write,
    process::{Command, Stdio},
};

use cloud_api::tencent;
use config::Config;
use cxx::{CxxVector, UniquePtr};
use qt_core::{q_init_resource, qs, ApplicationAttribute, QCoreApplication};
use qt_gui::QGuiApplication;
use qt_qml::{QQmlApplicationEngine, QQmlImageProviderBase};

#[cxx::bridge(namespace = "rust")]
mod ffi {
    extern "Rust" {
        fn ocr(img: UniquePtr<CxxVector<u8>>);
    }

    unsafe extern "C++" {
        include!("ocr/screenshot/rust/screen_image_provider.hpp");
        include!("ocr/screenshot/rust/message.hpp");
        type ScreenImageProvider;
        fn new_screen_image_provider() -> *const ScreenImageProvider;

        fn qml_register_message();
        fn qt_quit();
    }
}

async fn do_ocr(img: &[u8]) -> cloud_api::tencent::Result<()> {
    let base64img = base64::encode(img);
    let req = tencent::ocr::GeneralBasicOCRRequest::new(
        tencent::ocr::OCRImage::Base64(base64img),
        tencent::ocr::OCRLanguageType::Auto,
    );

    let cred = tencent::credential::Credential::new(
        String::from("AKIDoRoukKdfQv96mCLDo8CyThfLkskLfiV1"),
        String::from("FDuLVOzKQFn44nFQM1PWMvCCwPaU7UaP"),
    );

    let resp = tencent::api::post::<
        tencent::ocr::GeneralBasicOCRRequest,
        tencent::ocr::GeneralBasicOCRResponse,
    >(&req, &cred)
    .await?;

    let text = resp
        .text_detections
        .iter()
        .map(|td| &td.detected_text)
        .fold(String::new(), |lhs, rhs| lhs + rhs + "\n");

    println!("{}", text);

    let mut child = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");
    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    stdin
        .write_all(text.as_bytes())
        .expect("Failed to write to stdin");

    Ok(())
}

fn ocr(img: UniquePtr<CxxVector<u8>>) {
    tokio::spawn(async move {
        if let Err(e) = do_ocr(img.as_slice()).await {
            println!("{}", e);
        };

        ffi::qt_quit();
    });
}

pub fn run(config: Config) {
    println!("{:?}", config);
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

            ffi::qml_register_message();

            engine.load_q_string(&qs("qrc:/main.qml"));

            QGuiApplication::exec()
        }
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
