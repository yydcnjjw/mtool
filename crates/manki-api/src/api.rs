use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::{
    message::{Request, Response},
    Error,
};

const ANKI_LOCAL_URL: &str = "http://localhost:8765";

pub type InvokeResult<Response, Params> = Result<Response, Error<Params>>;

pub async fn invoke<Params, Result>(request: Request<Params>) -> InvokeResult<Result, Params>
where
    Params: Serialize + Debug + Send + 'static,
    Result: for<'a> Deserialize<'a> + Send + 'static,
{
    let Response::<Result> { result, error } = reqwest::Client::new()
        .post(ANKI_LOCAL_URL)
        .json(&request)
        .send()
        .await
        .context(format!("Failed to send {:?}", request))?
        .json()
        .await
        .context("Failed to parse json")?;

    if let Some(error) = error {
        return Err(Error::Invoke { request, error });
    }

    Ok(result)
}
