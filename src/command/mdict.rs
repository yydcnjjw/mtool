use std::{fs::File, io::Write};

use clap::Clap;
use mdict::{mdsearch::MdSearch, mdx::Mdx};

#[derive(Clap)]
pub struct Mdict {
    /// query
    #[clap(required(true), index(1))]
    query: String,
    /// dict path
    #[clap(short, long)]
    dict_path: String,
}

impl Mdict {
    pub async fn run(&self) {
        let mdx = Mdx::parse(&self.dict_path.into());

        match mdx {
            Ok((_, mdx)) => {
                let mut file = File::create("test.html").unwrap();
                mdx.search(self.query.clone())
                    .iter()
                    .map(|item| (item.0.clone(), format!("{:?} <div></div>", item.1)))
                    .for_each(|item| {
                        println!("{:?}", item);
                    });
                file.flush().unwrap();
            }
            Err(e) => println!("{:?}", e),
        }
    }
}
