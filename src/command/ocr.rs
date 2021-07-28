use std::{
    ffi::CString,
    os::raw::{c_char, c_int},
};

use clap::Clap;

#[derive(Clap)]
pub struct Ocr {}

impl Ocr {
    pub async fn run(&self) {
        let args = std::env::args()
            .map(|arg| CString::new(arg).unwrap())
            .collect::<Vec<CString>>();
        let c_args = args
            .iter()
            .map(|arg| arg.as_ptr())
            .collect::<Vec<*const c_char>>();

        unsafe {
            ocr::qt_run(c_args.len() as c_int, c_args.as_ptr());
        }
    }
}
