use std::os::unix::ffi::OsStrExt;
use std::{
    ffi::CString,
    os::raw::{c_char, c_int},
    ptr,
};

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("ocr/screenshot/screenshot.hpp");

        unsafe fn qt_run(argc: i32, argv: *mut *mut c_char, f: fn(UniquePtr<CxxVector<u8>>)) -> i32;
    }
}

pub fn run() {
    let args: Vec<CString> = std::env::args_os()
        .map(|os_str| {
            let bytes = os_str.as_bytes();
            CString::new(bytes).unwrap_or_else(|nul_error| {
                let nul_position = nul_error.nul_position();
                let mut bytes = nul_error.into_vec();
                bytes.truncate(nul_position);
                CString::new(bytes).unwrap()
            })
        })
        .collect();

    let argc = args.len();
    let mut argv: Vec<*mut c_char> = Vec::with_capacity(argc + 1);
    for arg in &args {
        argv.push(arg.as_ptr() as *mut c_char);
    }
    argv.push(ptr::null_mut()); // Nul terminator.

    unsafe {
        ffi::qt_run(argc as i32, argv.as_mut_ptr(), |img| {
            println!("{:?} test", img.len());
            // tokio::spawn(async move {
            //     cloud_api::tencent::run(size, data).await.unwrap();
            // });
        });
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
