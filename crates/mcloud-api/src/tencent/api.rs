use anyhow::Context;
use chrono::{DateTime, Utc};
use digest::Digest;
use hmac::{Hmac, Mac};
use reqwest::header::{CONTENT_TYPE, HOST};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use super::credential::Credential;
use super::{Error, Result};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ErrorResponse {
    pub error: ApiError,
    pub request_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
#[serde(rename_all = "PascalCase")]
pub enum ResponseType<T> {
    Ok(T),
    Err(ErrorResponse),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct HttpResponse<T> {
    pub response: ResponseType<T>,
}

pub trait HttpRequest {
    fn service() -> String;
    fn host() -> String;
    fn action() -> String;
    fn version() -> String;
}

fn make_client<T>(req: &T, cred: &Credential) -> Result<reqwest::RequestBuilder>
where
    T: HttpRequest + Serialize,
{
    let secret_id = &cred.secret_id;
    let secret_key = &cred.secret_key;
    let algorithm = &cred.algorithm;

    let service = T::service();
    let host = T::host();
    let endpoint = format!("https://{}", host);

    let action = T::action();
    let version = T::version();

    let utc: DateTime<Utc> = Utc::now();
    let timestamp = utc.timestamp();
    let date = utc.format("%Y-%m-%d").to_string();

    let method = "POST";
    let canonical_uri = "/";
    let canonical_querystring = "";
    let ct = "application/json; charset=utf-8";

    let payload = serde_json::to_string(&req).context("Failed to serialize request")?;

    let canonical_headers = format!("content-type:{}\nhost:{}\n", ct, host);
    let signed_headers = "content-type;host";

    let sha256 = |input| {
        let mut sha256 = Sha256::new();
        sha256.update(input);
        hex::encode(sha256.finalize())
    };

    let hashed_request_payload = sha256(&payload);

    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        method,
        canonical_uri,
        canonical_querystring,
        canonical_headers,
        signed_headers,
        hashed_request_payload
    );

    let credential_scope = format!("{}/{}/tc3_request", date, service);
    let hashed_canonical_request = sha256(&canonical_request);
    let string_to_sign = format!(
        "{}\n{}\n{}\n{}",
        algorithm, timestamp, credential_scope, hashed_canonical_request
    );

    let sign = |input, key| {
        type HmacSha256 = Hmac<Sha256>;
        let mut hmac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
        hmac.update(input);
        hmac.finalize().into_bytes()
    };

    let tc3_secret_key = format!("TC3{}", secret_key);
    let secret_date = sign(date.as_bytes(), tc3_secret_key.as_bytes());
    let secret_service = sign(service.as_bytes(), &secret_date);
    let secret_signing = sign("tc3_request".as_bytes(), &secret_service);
    let signature = hex::encode(&sign(string_to_sign.as_bytes(), &secret_signing));

    let authorization = format!(
        "{} Credential={}/{}, SignedHeaders={}, Signature={}",
        algorithm, secret_id, credential_scope, signed_headers, signature
    );

    Ok(reqwest::Client::new()
        .post(endpoint)
        .header(HOST, host)
        .header(CONTENT_TYPE, ct)
        .header("X-TC-Action", action)
        .header("X-TC-Version", version)
        .header("X-TC-Timestamp", timestamp)
        .header("X-TC-Region", "ap-shanghai")
        .header("Authorization", authorization)
        .body(payload))
}

pub async fn post<Request, Response>(req: &Request, cred: &Credential) -> Result<Response>
where
    Request: HttpRequest + Serialize,
    Response: DeserializeOwned,
{
    let cli = make_client(req, cred)?;

    match cli
        .send()
        .await
        .context("Failed to send request")?
        .json::<HttpResponse<Response>>()
        .await
        .context("Failed to deserialize resposne")?
        .response
    {
        ResponseType::Ok(resp) => Ok(resp),
        ResponseType::Err(e) => Err(Error::Api(e.error)),
    }
}
