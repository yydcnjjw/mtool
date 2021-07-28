use std::os::raw::{c_char, c_int};

#[link(name = "screenshot")]
extern "C" {
    pub fn qt_run(argc: c_int, argv: *const *const c_char) -> c_int;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
