use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::process::{Command, Stdio};
use std::{
    ffi::CString,
    os::raw::{c_char, c_int},
    ptr,
};

use cpp_core::{CastInto, Ptr, StaticUpcast};
use cxx::{CxxVector, UniquePtr};
use qt_core::{q_init_resource, qs, ApplicationAttribute, QCoreApplication, QFlags, QString};
use qt_gui::QGuiApplication;
use qt_qml::{QQmlApplicationEngine, QQmlImageProviderBase};

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("ocr/screenshot/screenshot.hpp");
        type QQmlImageProviderBase;

        type ScreenImageProvider;
        fn new_screen_image_provider() -> UniquePtr<QQmlImageProviderBase>;
        // unsafe fn qt_run(argc: i32, argv: *mut *mut c_char, f: fn(UniquePtr<CxxVector<u8>>))
        //     -> i32;
        // fn qt_quit();

    }
}

pub fn run() {
    unsafe {
        QCoreApplication::set_attribute_1a(ApplicationAttribute::AAEnableHighDpiScaling);
    }

    QGuiApplication::init(|_| -> i32 {
        unsafe {
            q_init_resource!("qml");
            let engine = QQmlApplicationEngine::from_q_string(&qs("qrc:/main.qml"));
            let provider = ffi::new_screen_image_provider();
            engine.add_image_provider(
                &qs("screen"),
                provider.into_raw(),
            );
            QGuiApplication::exec()
        }
    })

    // let args: Vec<CString> = std::env::args_os()
    //     .map(|os_str| {
    //         let bytes = os_str.as_bytes();
    //         CString::new(bytes).unwrap_or_else(|nul_error| {
    //             let nul_position = nul_error.nul_position();
    //             let mut bytes = nul_error.into_vec();
    //             bytes.truncate(nul_position);
    //             CString::new(bytes).unwrap()
    //         })
    //     })
    //     .collect();

    // let argc = args.len();
    // let mut argv: Vec<*mut c_char> = Vec::with_capacity(argc + 1);
    // for arg in &args {
    //     argv.push(arg.as_ptr() as *mut c_char);
    // }
    // argv.push(ptr::null_mut()); // Nul terminator.

    // unsafe {
    //     ffi::qt_run(
    //         argc as i32,
    //         argv.as_mut_ptr(),
    //         |img: UniquePtr<CxxVector<u8>>| {
    //             tokio::spawn(async move {
    //                 let text = cloud_api::tencent::run(img.as_slice())
    //                     .await
    //                     .unwrap()
    //                     .concat();

    //                 println!("{}", text);

    //                 let mut child = Command::new("xclip")
    //                     .arg("-selection")
    //                     .arg("clipboard")
    //                     .stdin(Stdio::piped())
    //                     .spawn()
    //                     .expect("Failed to spawn child process");

    //                 let mut stdin = child.stdin.take().expect("Failed to open stdin");
    //                 std::thread::spawn(move || {
    //                     stdin
    //                         .write_all(text.as_bytes())
    //                         .expect("Failed to write to stdin");
    //                 });
    //                 ffi::qt_quit();
    //             });
    //         },
    //     );
    // }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
