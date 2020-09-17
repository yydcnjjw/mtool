use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
struct AnkiRequest<T> {
    action: String,
    params: T,
    version: usize,
}

impl<T> AnkiRequest<T> {
    fn new(action: &str, params: T) -> AnkiRequest<T> {
        AnkiRequest {
            action: action.to_string(),
            params,
            version: 6,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnkiResponse {
    result: serde_json::Value,
    error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnkiNoteOptions {
    #[serde(rename = "allowDuplicate")]
    pub allow_duplicate: bool,
    #[serde(rename = "duplicateScope")]
    pub duplicate_scope: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnkiNote {
    #[serde(rename = "deckName")]
    pub deck_name: String,
    #[serde(rename = "modelName")]
    pub model_name: String,
    pub fields: serde_json::Value,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<AnkiNoteOptions>,
}

const API_URL: &str = "http://localhost:8765";

#[derive(Debug)]
pub enum Error {
    NetRequest(reqwest::Error),
    AnkiResponse(String),
}

type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            Error::NetRequest(e) => e.fmt(f),
            Error::AnkiResponse(s) => write!(f, "anki response: {}", s),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Error {
        Error::NetRequest(e)
    }
}

async fn api_request<T>(request: &AnkiRequest<T>) -> Result<AnkiResponse>
where
    T: Serialize,
{
    Ok(reqwest::Client::new()
        .post(API_URL)
        .json(request)
        .send()
        .await?
        .json()
        .await?)
}

pub async fn version() -> Result<AnkiResponse> {
    api_request(&AnkiRequest::new("version", serde_json::Map::new())).await
}

pub async fn can_add_note(note: &AnkiNote) -> Result<bool> {
    api_request(&AnkiRequest::new(
        "canAddNotes",
        serde_json::json!({ "notes": [note] }),
    ))
    .await
    .and_then(|v: AnkiResponse| {
        if v.error.is_none() {
            Ok(v.result
                .as_array()
                .unwrap()
                .get(0)
                .unwrap()
                .as_bool()
                .unwrap())
        } else {
            Err(Error::AnkiResponse(v.error.unwrap()))
        }
    })
}

pub async fn add_note(note: &AnkiNote) -> Result<u64> {
    api_request(&AnkiRequest::new(
        "addNote",
        serde_json::json!({ "note": note }),
    ))
    .await
    .and_then(|v: AnkiResponse| {
        if v.error.is_none() {
            Ok(v.result.as_u64().unwrap())
        } else {
            Err(Error::AnkiResponse(v.error.unwrap()))
        }
    })
}
