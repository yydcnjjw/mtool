use eventsource_stream::Eventsource;
use reqwest::header::{self, ACCEPT, ACCEPT_LANGUAGE, COOKIE, ORIGIN, REFERER, USER_AGENT};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Options {
    pub skill: String,
    pub date: String,
    pub language: String,
    pub detailed: bool,
    pub creative: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub question: String,
    pub bing_results: Option<String>,
    pub code_context: String,
    pub options: Options,
}

async fn run() -> Result<(), anyhow::Error> {
    let cf_clearnce = "4GyC.mAIxS209d18D.LK2Iil7YOJMLUAkLBr2W9o4V8-1682585029-0-250";
    let user_agent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36 Edg/113.0.0.0";

    let mut headers = header::HeaderMap::new();
    headers.insert("authority", "www.phind.com".parse()?);
    headers.insert(ACCEPT, "*/*".parse()?);
    headers.insert(
        ACCEPT_LANGUAGE,
        "en,fr-FR;q=0.9,fr;q=0.8,es-ES;q=0.7,es;q=0.6,en-US;q=0.5,am;q=0.4,de;q=0.3".parse()?,
    );
    headers.insert("content-type", "application/json".parse()?);

    headers.insert(ORIGIN, "https://www.phind.com".parse()?);
    headers.insert(
        REFERER,
        "https://www.phind.com/search?q=hi&c=&source=searchbox&init=true".parse()?,
    );
    headers.insert(
        "sec-ch-ua",
        r#""Chromium";v="112", "Google Chrome";v="112", "Not:A-Brand";v="99""#.parse()?,
    );
    headers.insert("sec-ch-ua-mobile", "?0".parse()?);
    headers.insert("sec-ch-ua-platform", r#""macOS""#.parse()?);
    headers.insert("sec-fetch-dest", "empty".parse()?);
    headers.insert("sec-fetch-mode", "cors".parse()?);
    headers.insert("sec-fetch-site", "same-origin".parse()?);
    headers.insert(COOKIE, format!("cf_clearance={}", cf_clearnce).parse()?);
    headers.insert(USER_AGENT, user_agent.parse()?);

    let cli = reqwest::ClientBuilder::default()
        .default_headers(headers)
        .build()?;

    let mut stream = cli
        .post("https://www.phind.com/api/infer/answer")
        .json(&Request {
            question: "Hello".into(),
            bing_results: None,
            code_context: "".into(),
            options: Options {
                skill: "expert".into(),
                date: chrono::Utc::now().format("%d/%m/%Y").to_string(),
                language: "en".into(),
                detailed: false,
                creative: false,
            },
        })
        .send()
        .await?
        .bytes_stream()
        .eventsource();

    while let Some(e) = stream.next().await {
        println!("{:?}", e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_phind() -> Result<(), anyhow::Error> {
        run().await
    }
}
