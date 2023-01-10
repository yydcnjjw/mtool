mod output;
mod parser;
mod sense;
mod synonym;
mod thesaures;

use reqwest::header::USER_AGENT;

const DEFAULT_USER_AGENT: &'static str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_13_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/60.0.3112.78 Safari/537.36";

pub use self::{sense::Sense, synonym::Synonym, thesaures::ThesauresResult};

pub async fn query(word: &str) -> Result<Vec<ThesauresResult>, anyhow::Error> {
    let cli = reqwest::Client::builder()
        .use_rustls_tls() // for tls fingerpint
        .build()?;

    let resp = cli
        .get(&format!(
            "https://www.collinsdictionary.com/dictionary/english-thesaurus/{}",
            word
        ))
        .header(USER_AGENT, DEFAULT_USER_AGENT)
        .send()
        .await?
        .text()
        .await?;

    parser::parse(&resp)
}

#[cfg(test)]
mod tests {
    use crate::decode::collins::output::OutputOrg;

    use super::query;

    #[tokio::test]
    async fn test_thesaures() {
        let result = query("test").await.unwrap();
        let mut output = String::new();
        for v in result {
            v.output_org(&mut output).unwrap();
        }

        println!("{}", output);
    }
}
