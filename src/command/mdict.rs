use std::{io::Cursor, path::Path};

use clap::Clap;

use mdict::common::MdResource;

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
        match mdict::parse(&Path::new(&self.dict_path)) {
            Ok(mut md) => {
                md.search(&self.query)
                    .iter()
                    .filter_map(|item| {
                        let text = match &item.1 {
                            MdResource::Text(text) => text,
                            _ => {
                                return None;
                            }
                        };
                        Some((item.0.clone(), format!("<div>{} ----------</div>{}", item.0, text)))
                    })
                    .for_each(|item| {
                        println!("{}", html2text::from_read(Cursor::new(&item.1), 100));
                    });
            }
            Err(e) => println!("{:?}", e),
        }
    }
}
