use std::{
    io::Write,
    process::{Command, Stdio},
};

use cxx::{CxxVector, UniquePtr};
use qt_core::{q_init_resource, qs, ApplicationAttribute, QCoreApplication};
use qt_gui::QGuiApplication;
use qt_qml::{QQmlApplicationEngine, QQmlImageProviderBase};

#[cxx::bridge(namespace = "rust")]
mod ffi {
    extern "Rust" {
        fn ocr_test(img: UniquePtr<CxxVector<u8>>);
    }

    unsafe extern "C++" {
        include!("ocr/screenshot/rust/screen_image_provider.hpp");
        include!("ocr/screenshot/rust/message.hpp");
        type ScreenImageProvider;
        fn new_screen_image_provider() -> *const ScreenImageProvider;

        fn qml_register_message();
    }
}

fn ocr_test(img: UniquePtr<CxxVector<u8>>) {
    tokio::spawn(async move {
        let text = cloud_api::tencent::run(img.as_slice())
            .await
            .unwrap()
            .concat();

        println!("{}", text);

        let mut child = Command::new("xclip")
            .arg("-selection")
            .arg("clipboard")
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed to spawn child process");

        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        std::thread::spawn(move || {
            stdin
                .write_all(text.as_bytes())
                .expect("Failed to write to stdin");
        });
    });
}

pub fn run() {
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

            engine.load_q_string(&qs("qrc:/main.qml"));

            ffi::qml_register_message();

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
