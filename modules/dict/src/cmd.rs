use anyhow::Context;
use async_trait::async_trait;
use clap::Parser;
use cmder_mod::Command;
use mdict::decode::collins::{self, output::OutputOrg};

#[derive(Debug)]
pub enum DictCmd {
    CollinsDict,
    CollinsThsaures,
}

/// Dict module
#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    /// query
    query: String,
}

impl DictCmd {
    async fn execute(&mut self, args: Vec<String>) -> anyhow::Result<()> {
        let Args { query } = Args::try_parse_from(args).context("Failed to parse dict args")?;

        let result: Vec<String> = match self {
            DictCmd::CollinsDict => collins::dict::query(&query)
                .await?
                .iter()
                .filter_map(|v| v.to_string_org().ok())
                .collect(),
            DictCmd::CollinsThsaures => collins::thesaures::query(&query)
                .await?
                .iter()
                .filter_map(|v| v.to_string_org().ok())
                .collect(),
        };

        if result.is_empty() {
            println!("Failed to query {}", query);
        } else {
            for item in result {
                println!("{}", item);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Command for DictCmd {
    async fn exec(&mut self, args: Vec<String>) {
        if let Err(e) = self.execute(args).await {
            log::warn!("{:?}", e);
        }
    }
}
