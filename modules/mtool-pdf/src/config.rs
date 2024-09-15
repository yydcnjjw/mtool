use mapp::serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct AdobeApiConfig {
    pub url: String,
    pub client_id: String,
    pub key: String,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct Config {
    pub pdfium: String,
    pub adobe_api: AdobeApiConfig,
}
