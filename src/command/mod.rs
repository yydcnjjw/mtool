pub mod dict;
pub mod mdict;
pub mod ocr;
pub mod search;
pub mod translate;

use self::{dict::DictOpt, mdict::Mdict, ocr::Ocr, search::Search, translate::TranslateOpt};

use clap::Clap;

#[derive(Clap)]
pub enum SubCommand {
    /// dict
    #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    Dict(DictOpt),
    /// translate
    #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    Translate(TranslateOpt),
    /// search
    #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    Search(Search),
    /// ocr
    #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    Ocr(Ocr),
    /// mdict
    #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
    Mdict(Mdict),
}
