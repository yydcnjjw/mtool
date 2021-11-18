use serde::{Deserialize, Serialize};

use crate::tencent::api::HttpRequest;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum LanguageType {
    Auto,
    Zh,
    Ja,
    En,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct TextTranslateRequest {
    source_text: String,
    source: LanguageType,
    target: LanguageType,
    project_id: i32,
    untranslated_text: String,
}

impl TextTranslateRequest {
    pub fn new(source_text: String, source: LanguageType, target: LanguageType) -> Self {
        Self {
            source_text,
            source,
            target,
            project_id: 0,
            untranslated_text: "".to_string(),
        }
    }
}

impl HttpRequest for TextTranslateRequest {
    fn service() -> String {
        String::from("tmt")
    }

    fn host() -> String {
        String::from("tmt.tencentcloudapi.com")
    }

    fn action() -> String {
        String::from("TextTranslate")
    }

    fn version() -> String {
        String::from("2018-03-21")
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct TextTranslateResponse {
    pub target_text: String,
    pub source: LanguageType,
    pub target: LanguageType,
    pub request_id: String,
}
