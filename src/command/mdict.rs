use std::{
    fs::File,
    io::{Read, Write},
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

        let mdx = mdict::mdx::parse(buf.as_slice());

        match mdx {
            Ok((_, mdx)) => {
                println!("query: {}", self.query);
                let mut file = File::create("test.html").unwrap();
                mdx.search(self.query.clone())
                    .iter()
                    .map(|item| (item.0.clone(), format!("{} <div></div>", item.1)))
                    .for_each(|item| {
                        file.write(item.1.as_bytes()).unwrap();
                    });
                file.flush().unwrap();
            }
            Err(e) => println!("{:?}", e),
        }
    }
}
