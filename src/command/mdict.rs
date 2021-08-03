use std::{
    fs::File,
    io::{Cursor, Read},
};

use clap::Clap;

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
        let mut file = File::open(&self.dict_path).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();

        let mdx = mdict::mdx::parse(&buf);

        match mdx {
            Ok((_, mdx)) => {
                println!("query: {}", self.query);
                mdx.search(self.query.clone())
                    .iter()
                    .map(|item| {
                        (
                            item.0.clone(),
                            html2text::from_read(Cursor::new(&item.1), 100),
                        )
                    })
                    .for_each(|item| {
                        println!("{}", item.1);
                    })
            }
            Err(e) => println!("{:?}", e),
        }
    }
}
