use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AdobeApiConfig {
    pub url: String,
    pub client_id: String,
    pub key: String,
}


#[derive(Debug, Deserialize)]
pub struct Config {
    pub pdfium: String,
    pub adobe_api: AdobeApiConfig,
}
