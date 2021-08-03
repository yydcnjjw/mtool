use clap::Clap;

#[derive(Clap)]
pub struct Ocr {}

impl Ocr {
    pub async fn run(&self) {
        ocr::run()
    }
}
