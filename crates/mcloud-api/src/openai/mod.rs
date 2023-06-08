use std::fmt::Debug;

use anyhow::Context;
use serde::{de::DeserializeOwned, Serialize};

pub mod chat;

pub trait RequestMeta {
    fn path() -> &'static str;
    fn api_name() -> &'static str;
    fn method() -> reqwest::Method;
}

pub struct Client {
    inner: reqwest::Client,
    basic_url: String,
    key: String,
}

impl Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("basic_url", &self.basic_url)
            .field("key", &self.key)
            .finish()
    }
}

impl Client {
    pub fn new(url: &str, key: &str) -> Result<Self, anyhow::Error> {
        Ok(Self {
            inner: reqwest::ClientBuilder::new().build()?,
            basic_url: url.into(),
            key: key.into(),
        })
    }

    pub async fn send<Request, Response>(&self, req: &Request) -> Result<Response, anyhow::Error>
    where
        Request: RequestMeta + Serialize + Debug,
        Response: DeserializeOwned,
    {
        self.inner
            .request(
                Request::method(),
                format!("{}/{}", self.basic_url, Request::path()),
            )
            .header("Authorization", format!("Bearer {}", self.key))
            .json(req)
            .send()
            .await
            .context(format!("{} send failed: {:?}", Request::api_name(), req))?
            .json()
            .await
            .context(format!("{} recv failed", Request::api_name()))
    }
}
