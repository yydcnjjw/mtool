use mapp::{
    anyhow,
    regex::Regex,
    reqwest,
    serde::{Deserialize, Serialize},
};
use std::time::Duration;

#[derive(Debug)]
pub struct LyricEntry {
    pub timestamp: Duration,
    pub text: String,
}

#[derive(Debug)]
pub struct Lyric {
    pub entries: Vec<LyricEntry>,
}

pub async fn get_netease_lyrics(id: String) -> Result<Option<Lyric>, anyhow::Error> {
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(crate = "mapp::serde")]
    #[serde(rename_all = "camelCase")]
    struct LyricResponse {
        sgc: bool,
        sfy: bool,
        qfy: bool,
        trans_user: Option<User>,
        lyric_user: Option<User>,
        lrc: Option<Lrc>,
        code: u32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(crate = "mapp::serde")]
    #[serde(rename_all = "camelCase")]
    struct User {
        id: u32,
        status: u8,
        demand: u8,
        userid: u64,
        nickname: String,
        uptime: u64,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(crate = "mapp::serde")]
    struct Lrc {
        version: u32,
        lyric: String,
    }

    fn parse_lyrics(text: &str) -> Lyric {
        let re = Regex::new(r"\[(\d{2}):(\d{2})\.(\d{2,3})\](.*)").unwrap();
        let mut entries = Vec::new();

        for line in text.lines() {
            if let Some(captures) = re.captures(line) {
                let minutes: u64 = captures[1].parse().unwrap_or(0);
                let seconds: u64 = captures[2].parse().unwrap_or(0);
                let mut milliseconds: u64 = captures[3].parse().unwrap_or(0);
                if captures[3].len() == 2 {
                    milliseconds *= 10;
                }

                let timestamp = Duration::from_secs(minutes * 60 + seconds)
                    + Duration::from_millis(milliseconds);
                let text = captures[4].trim().to_string();

                if !text.is_empty() {
                    entries.push(LyricEntry { timestamp, text });
                }
            }
        }

        Lyric { entries }
    }

    let data: LyricResponse = reqwest::get(format!(
        "https://music.163.com/api/song/lyric?os=pc&id={}&lv=-1",
        id
    ))
    .await?
    .json()
    .await?;

    Ok(data.lrc.map(|Lrc { version, lyric }| parse_lyrics(&lyric)))
}
