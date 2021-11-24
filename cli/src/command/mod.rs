mod translate;
mod test;

use std::collections::HashMap;

use anyhow::Context;
use async_trait::async_trait;

use crate::app::App;

type Cmd = Box<dyn Command>;

pub struct Commander {
    cmds: HashMap<String, Cmd>,
}

impl Commander {
    pub fn new() -> Self {
        Self {
            cmds: HashMap::new(),
        }
    }

    pub async fn exec(&mut self, name: &String, args: &[String]) -> anyhow::Result<()> {
        let cmd = self
            .get(name)
            .with_context(|| format!("Command `{}` not found", name))?;
        cmd.exec(args.to_vec()).await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn list_command_name(&self) -> Vec<&String> {
        self.cmds.keys().collect::<_>()
    }

    pub fn get(&mut self, name: &String) -> Option<&mut Cmd> {
        self.cmds.get_mut(name)
    }

    pub fn insert(&mut self, name: String, cmd: Cmd) {
        self.cmds.insert(name, cmd);
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, name: String) {
        self.cmds.remove(&name);
    }
}

#[async_trait]
pub trait Command {
    async fn exec(&mut self, args: Vec<String>) -> anyhow::Result<()>;
}

pub fn add_command(app: &mut App) -> anyhow::Result<()> {
    translate::add_command(app)?;
    test::add_command(app)?;
    Ok(())
}

// pub mod dict;
// pub mod mdict;
// pub mod ocr;
// pub mod search;

// use self::{dict::DictCmd, mdict::MdictCmd, ocr::OcrCmd, search::SearchCmd, translate::TranslateCmd};

// use enum_dispatch::enum_dispatch;
// use mytool_core::app::{App, Result};

// use clap::Clap;
// use log::info;

// use super::opts::Opts;

// #[derive(Clap)]
// #[enum_dispatch]
// pub enum SubCommand {
//     // test
//     #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
//     Test(TestCmd),
//     // /// dict
//     // #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
//     // Dict(DictCmd),
//     // /// translate
//     // #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
//     // Translate(TranslateCmd),
//     // /// search
//     // #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
//     // Search(SearchCmd),
//     // /// ocr
//     // #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
//     // Ocr(OcrCmd),
//     // /// mdict
//     // #[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
//     // Mdict(MdictCmd),
// }

// impl SubCommand {
//     pub async fn exec(&self, app: &App<Opts>) -> Result<()> {
//         self.run(app).await
//     }
// }

// #[async_trait]
// #[enum_dispatch(SubCommand)]
// pub trait CommandRunner {
//     async fn run(&self, app: &App<Opts>) -> Result<()>;
// }

// #[derive(Clap)]
// pub struct TestCmd {}

// #[async_trait]
// impl CommandRunner for TestCmd {
//     async fn run(&self, _app: &App<Opts>) -> Result<()> {
//         info!("test");
//         Ok(())
//     }
// }
