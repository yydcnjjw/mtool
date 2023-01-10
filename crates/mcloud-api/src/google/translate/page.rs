use anyhow::Context;
use thiserror::Error;

const GOOGLE_TRANSLATE_ROOT_URL: &str = "https://translate.google.com/";
const GOOGLE_TRANSLATE_API_URL: &str = "https://translate.google.com/translate_a/single";
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, \
                          like Gecko) Chrome/84.0.4147.89 Safari/537.36";
type TKK = (usize, usize);

#[derive(Error, Debug)]
pub enum Error {
    #[error("TKK not found")]
    TKKNotFound,
    #[error("request parse failure")]
    RequestParse,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

async fn tkk() -> Result<TKK> {
    let resp = reqwest::Client::new()
        .get(GOOGLE_TRANSLATE_ROOT_URL)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await
        .context(format!(
            "Failed to send request {}",
            GOOGLE_TRANSLATE_ROOT_URL
        ))?
        .text()
        .await
        .context("Failed to recv response")?;
    match regex::Regex::new(r"tkk:'(\d*).(\d*)'")
        .unwrap()
        .captures_iter(&resp)
        .map(|v: regex::Captures| {
            (
                v[1].parse::<usize>().unwrap(),
                v[2].parse::<usize>().unwrap(),
            )
        })
        .nth(0)
    {
        Some(v) => Ok(v),
        None => Err(Error::TKKNotFound),
    }
}

fn wr(mut a: usize, b: &str) -> usize {
    let b: Vec<usize> = b.chars().map(|c| c as usize).collect();
    let mut i = 0;
    while i < b.len() - 2 {
        let c = b[i + 2];
        let mut d = if 'a' as usize <= c {
            c - 87
        } else {
            c - '0' as usize
        };
        d = if '+' as usize == b[i + 1] {
            a >> d
        } else {
            a << d
        };
        a = if '+' as usize == b[i] {
            a + d & 4294967295
        } else {
            a ^ d
        };

        i += 3;
    }
    return a;
}

async fn tk(s: &str) -> Result<String> {
    let mut v = Vec::<usize>::new();
    let s: Vec<usize> = s.chars().map(|c| c as usize).collect();

    let mut i = 0;
    while i < s.len() {
        let mut c = s[i];
        if 128 > c {
            v.push(c);
        } else {
            if 2048 > c {
                v.push(c >> 6 | 192);
            } else {
                if 0xd800 == (c & 0xfc00) && i + 1 < s.len() && 0xdc00 == (s[i + 1] & 0xfc00) {
                    i += 1;
                    c = 0x10000 + ((c & 0x3ff) << 10) + (s[i] & 0x3ff);
                    v.push(c >> 18 | 240);
                    v.push((c >> 12 & 63) | 128);
                } else {
                    v.push(c >> 12 | 224);
                    v.push((c >> 6 & 63) | 128);
                }
            }
            v.push((c & 0x3f) | 0x80);
        }
        i += 1;
    }

    let tkk = tkk().await.unwrap_or((427116, 3269864724));

    let b = tkk.0;
    let mut a = b;

    let s1 = "+-a^+6";
    let s2 = "+-3^+b+-f";
    for i in v {
        a += i;
        a = wr(a, s1);
    }
    a = wr(a, s2);
    a ^= tkk.1;
    if (a as isize) < 0 {
        a = (a & 0x7fffffff) + 0x80000000;
    }
    a %= 1000000;
    Ok(format!("{}.{}", a, a ^ b))
}

fn simple(v: &serde_json::Value, i: usize) -> Option<String> {
    Some(
        v.as_array()?
            .get(0)?
            .as_array()?
            .iter()
            .fold(String::new(), |v1, v2| {
                v1 + v2
                    .as_array()
                    .unwrap()
                    .get(i)
                    .unwrap()
                    .as_str()
                    .unwrap_or("")
            }),
    )
}

fn simple_translation(v: &serde_json::Value) -> Option<String> {
    simple(v, 0)
}

pub async fn query(q: &str, from: &str, to: &str) -> Result<String> {
    let tk = tk(q).await.unwrap();
    let resp: serde_json::Value = reqwest::Client::new()
        .get(GOOGLE_TRANSLATE_API_URL)
        .query(&[
            ("client", "webapp"),
            ("sl", from),
            ("tl", to),
            ("hl", to),
            ("dt", "at"),
            ("dt", "bd"),
            ("dt", "ex"),
            ("dt", "ld"),
            ("dt", "md"),
            ("dt", "qca"),
            ("dt", "rw"),
            ("dt", "rm"),
            ("dt", "ss"),
            ("dt", "t"),
            ("tk", &tk),
            ("q", q),
        ])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    simple_translation(&resp).ok_or(Error::RequestParse)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test() {
        query("english", "auto", "zh-CN").await.unwrap()
    }
}
