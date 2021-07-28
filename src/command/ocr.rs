use clap::Clap;

#[derive(Clap)]
pub struct Ocr {}


impl Ocr {
    pub fn run(&self) {
        ocr::run()
    }
}
