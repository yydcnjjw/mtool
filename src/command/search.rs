use clap::Clap;

#[derive(Clap)]
pub struct Search {}

impl Search {
    pub async fn run(&self) {
        search::run();
    }
}
