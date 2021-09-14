// pub mod dict;
// pub mod mdict;
// pub mod ocr;
// pub mod search;
// pub mod translate;

// use self::{dict::DictCmd, mdict::MdictCmd, ocr::OcrCmd, search::SearchCmd, translate::TranslateCmd};

use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use mytool_core::app::{App, Result};

use clap::Clap;
use log::info;

use super::opts::Opts;

#[derive(Clap)]
#[enum_dispatch]
pub enum SubCommand {
    // test
    #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    Test(TestCmd),
    // /// dict
    // #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    // Dict(DictCmd),
    // /// translate
    // #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    // Translate(TranslateCmd),
    // /// search
    // #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    // Search(SearchCmd),
    // /// ocr
    // #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    // Ocr(OcrCmd),
    // /// mdict
    // #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    // Mdict(MdictCmd),
}

impl SubCommand {
    pub async fn exec(&self, app: &App<Opts>) -> Result<()> {
        self.run(app).await
    }
}

#[async_trait]
#[enum_dispatch(SubCommand)]
pub trait CommandRunner {
    async fn run(&self, app: &App<Opts>) -> Result<()>;
}

#[derive(Clap)]
pub struct TestCmd {}

#[async_trait]
impl CommandRunner for TestCmd {
    async fn run(&self, _app: &App<Opts>) -> Result<()> {
        info!("test");
        Ok(())
    }
}
