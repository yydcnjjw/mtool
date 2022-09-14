// use hjdict::parser::JPWord;
// use hjdict::parser::OutputFormat;

// use serde::{Deserialize, Serialize};
// use thiserror::Error;

// #[derive(Serialize, Deserialize, Debug)]
// struct AnkiRequest<T> {
//     action: String,
//     params: T,
//     version: usize,
// }

// impl<T> AnkiRequest<T> {
//     fn new(action: &str, params: T) -> AnkiRequest<T> {
//         AnkiRequest {
//             action: action.to_string(),
//             params,
//             version: 6,
//         }
//     }
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct AnkiResponse {
//     result: serde_json::Value,
//     error: Option<String>,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct AnkiNoteOptions {
//     #[serde(rename = "allowDuplicate")]
//     pub allow_duplicate: bool,
//     #[serde(rename = "duplicateScope")]
//     pub duplicate_scope: String,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct AnkiNote {
//     #[serde(rename = "deckName")]
//     pub deck_name: String,
//     #[serde(rename = "modelName")]
//     pub model_name: String,
//     pub fields: serde_json::Value,
//     pub tags: Vec<String>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub options: Option<AnkiNoteOptions>,
// }

// impl AnkiNote {
//     pub fn new(word_info: &JPWord, options: Option<AnkiNoteOptions>) -> AnkiNote {
//         AnkiNote {
//             deck_name: "Japanese_Word".to_string(),
//             model_name: "japanese(dict)".to_string(),
//             fields: serde_json::json!({
//                         "expression": word_info.expression,
//                         "pronounce": word_info.pronounce.pronounce,
//                         "kata": word_info.pronounce.kata,
//                         "tone": word_info.pronounce.tone,
//                         "audio": format!("[sound:{}]", word_info.pronounce.audio),
//                         "simple": if word_info.simples.is_empty() {
//                             word_info.details.to_html()
//                         } else {
//                             word_info.simples.to_html()
//                         } ,
//                         "sentence": word_info.details.to_html()

//             }),
//             tags: vec!["japanese(dict)".to_string()],
//             options,
//         }
//     }
// }

// const API_URL: &str = "http://localhost:8765";

// #[derive(Error, Debug)]
// pub enum Error {
//     #[error("anki response: {0}")]
//     AnkiResponse(String),
//     #[error("{0}")]
//     NetRequest(#[from] reqwest::Error),
// }

// type Result<T> = std::result::Result<T, Error>;

// async fn api_request<T>(request: &AnkiRequest<T>) -> Result<AnkiResponse>
// where
//     T: Serialize,
// {
//     Ok(reqwest::Client::new()
//         .post(API_URL)
//         .json(request)
//         .send()
//         .await?
//         .json()
//         .await?)
// }

// pub async fn version() -> Result<AnkiResponse> {
//     api_request(&AnkiRequest::new("version", serde_json::Map::new())).await
// }

// pub async fn can_add_note(note: &AnkiNote) -> Result<bool> {
//     api_request(&AnkiRequest::new(
//         "canAddNotes",
//         serde_json::json!({ "notes": [note] }),
//     ))
//     .await
//     .and_then(|v: AnkiResponse| {
//         if v.error.is_none() {
//             Ok(v.result
//                 .as_array()
//                 .unwrap()
//                 .get(0)
//                 .unwrap()
//                 .as_bool()
//                 .unwrap())
//         } else {
//             Err(Error::AnkiResponse(v.error.unwrap()))
//         }
//     })
// }

// pub async fn add_note(note: &AnkiNote) -> Result<u64> {
//     api_request(&AnkiRequest::new(
//         "addNote",
//         serde_json::json!({ "note": note }),
//     ))
//     .await
//     .and_then(|v: AnkiResponse| {
//         if v.error.is_none() {
//             Ok(v.result.as_u64().unwrap())
//         } else {
//             Err(Error::AnkiResponse(v.error.unwrap()))
//         }
//     })
// }
