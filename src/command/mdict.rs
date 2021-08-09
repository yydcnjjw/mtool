use std::path::Path;

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
        match mdict::parse(&Path::new(&self.dict_path)) {
            Ok(mut mdx) => {
                mdx.search(&self.query)
                    .iter()
                    .map(|item| (item.0.clone(), format!("{:?} <div></div>", item.1)))
                    .for_each(|item| {
                        println!("{:?}", item);
                    });
            }
            Err(e) => println!("{:?}", e),
        }
    }
}
