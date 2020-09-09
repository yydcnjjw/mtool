use async_trait::async_trait;
use cloud_api::tencent;

use crate::{config::Config, Result};

#[async_trait]
pub trait OCRConvert {
    async fn convert(&self, img: &[u8]) -> Result<Vec<String>>;
}

pub struct TencentConvertor {
    config: Config,
}

impl TencentConvertor {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[async_trait]
impl OCRConvert for TencentConvertor {
    async fn convert(&self, img: &[u8]) -> Result<Vec<String>> {
        let base64img = base64::encode(img);
        let req = tencent::ocr::GeneralBasicOCRRequest::new(
            tencent::ocr::OCRImage::Base64(base64img),
            tencent::ocr::OCRLanguageType::Auto,
        );

        let resp = tencent::api::post::<
            tencent::ocr::GeneralBasicOCRRequest,
            tencent::ocr::GeneralBasicOCRResponse,
        >(
            &req,
            &tencent::credential::Credential::from(self.config.credential.clone()),
        )
        .await?;

        Ok(resp
            .text_detections
            .iter()
            .map(|td| td.detected_text.clone())
            .collect::<_>())
    }
}
