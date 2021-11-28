mod x11;

pub use self::x11::run_loop;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_loop() {
        run_loop(|e| {
            println!("{:?}", e);
        })
        .unwrap();
    }
}
