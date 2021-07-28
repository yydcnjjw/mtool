use chrono::{DateTime, Utc};
use crypto::{digest::Digest, hmac::Hmac, mac::Mac, sha2::Sha256};
use reqwest::header::{CONTENT_TYPE, HOST};
use serde::{Deserialize, Serialize};

use super::credential::Credential;

#[derive(Deserialize, Debug)]
pub struct HttpResponse<T> {
    #[serde(rename = "Response")]
    pub response: T,
}

pub trait HttpRequest {
    fn service() -> String;
    fn host() -> String;
    fn action() -> String;
    fn version() -> String;
}

pub fn make_client<T: HttpRequest + Serialize>(
    req: T,
    cred: Credential,
) -> reqwest::RequestBuilder {
    let secret_id = cred.secret_id;
    let secret_key = cred.secret_key;
    let algorithm = cred.algorithm;

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

    let payload = serde_json::to_string(&req).unwrap();

    let canonical_headers = format!("content-type:{}\nhost:{}\n", ct, host);
    let signed_headers = "content-type;host";

    let sha256 = |input| {
        let mut sha256 = Sha256::new();
        sha256.input_str(input);
        sha256.result_str().to_lowercase()
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
        let mut hmac = Hmac::new(Sha256::new(), key);
        hmac.input(input);
        hmac.result()
    };

    let tc3_secret_key = format!("TC3{}", secret_key);
    let secret_date = sign(date.as_bytes(), tc3_secret_key.as_bytes());
    let secret_service = sign(service.as_bytes(), secret_date.code());
    let secret_signing = sign("tc3_request".as_bytes(), secret_service.code());
    let signature = hex::encode(sign(string_to_sign.as_bytes(), secret_signing.code()).code());

    let authorization = format!(
        "{} Credential={}/{}, SignedHeaders={}, Signature={}",
        algorithm, secret_id, credential_scope, signed_headers, signature
    );

    reqwest::Client::new()
        .post(endpoint)
        .header(HOST, host)
        .header(CONTENT_TYPE, ct)
        .header("X-TC-Action", action)
        .header("X-TC-Version", version)
        .header("X-TC-Timestamp", timestamp)
        .header("X-TC-Region", "ap-shanghai")
        .header("Authorization", authorization)
        .body(payload)
}
