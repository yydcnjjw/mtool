use reqwest;

use crate::{
    parser::{parse, JPWord},
    Result,
};

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) \
                          AppleWebKit/537.36 (KHTML, like Gecko) \
                          Chrome/69.0.3497.81 Safari/537.36";
const COOKIE: &str = "HJ_UID=0f406091-be97-6b64-f1fc-f7b2470883e9; \
                      HJ_CST=1; HJ_CSST_3=1; TRACKSITEMAP=3%2C; \
                      HJ_SID=393c85c7-abac-f408-6a32-a1f125d7e8c6; \
                      _REF=; HJ_SSID_3=4a460f19-c0ae-12a7-8e86-6e360f69ec9b; \
                      _SREF_3=; HJ_CMATCH=1";

const API_URL: &str = "https://dict.hjenglish.com/jp/jc/";

pub async fn query_jp_dict(input: &String) -> Result<Vec<JPWord>> {
    let resp = reqwest::Client::new()
        .get(&format!("{}{}", API_URL, input))
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::COOKIE, COOKIE)
        .send()
        .await?;

    let text = resp.text().await?;

    parse(input, &text)
}
