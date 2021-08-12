pub mod dict;
pub mod mdict;
pub mod ocr;
pub mod search;
pub mod translate;

use self::{dict::DictCmd, mdict::MdictCmd, ocr::OcrCmd, search::SearchCmd, translate::TranslateCmd};
use crate::{app::App, error::Result};
use async_trait::async_trait;

use clap::Clap;

#[derive(Clap)]
pub enum SubCommand {
    /// dict
    #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    Dict(DictCmd),
    /// translate
    #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    Translate(TranslateCmd),
    /// search
    #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    Search(SearchCmd),
    /// ocr
    #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    Ocr(OcrCmd),
    /// mdict
    #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    Mdict(MdictCmd),
}

#[async_trait]
pub trait CommandRunner {
    async fn run(&self, app: &App) -> Result<()>;
}
