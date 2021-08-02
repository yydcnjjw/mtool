use std::convert::AsRef;
use thiserror::Error;

use self::{
    api::{make_client, ApiError, HttpResponse},
    credential::Credential,
    ocr::{GeneralBasicOCRResponse, OCRImage, OCRLanguageType},
};

mod api;
mod credential;
mod ocr;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0:?}")]
    Api(ApiError),
    #[error("{0}")]
    NetRequest(#[from] reqwest::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub async fn run<T>(img: T) -> Result<Vec<String>>
where
    T: AsRef<[u8]>,
{
    let base64img = base64::encode(img);
    let req = ocr::GeneralBasicOCRRequest::new(OCRImage::Base64(base64img), OCRLanguageType::Auto);
    let cred = Credential::new(
        String::from("AKIDoRoukKdfQv96mCLDo8CyThfLkskLfiV1"),
        String::from("FDuLVOzKQFn44nFQM1PWMvCCwPaU7UaP"),
    );
    let cli = make_client(req, cred);

    match cli
        .send()
        .await?
        .json::<HttpResponse<GeneralBasicOCRResponse>>()
        .await?
        .response
    {
        api::ResponseType::Ok(resp) => Ok(resp
            .text_detections
            .iter()
            .map(|td| td.detected_text.clone())
            .collect::<Vec<String>>()),
        api::ResponseType::Err(e) => Err(Error::Api(e.error)),
    }
}

#[cfg(test)]
mod tests {
    use super::{api::HttpResponse, ocr::GeneralBasicOCRResponse};

    #[test]
    fn de_ok() {
        assert!(
            serde_json::from_str::<HttpResponse<GeneralBasicOCRResponse>>(
                r#"
{
    "Response": {
        "TextDetections": [
            {
                "DetectedText": "https://stackoverflow.com » questions > how-t... t",
                "Confidence": 96,
                "ItemPolygon": {
                    "X": 33,
                    "Y": 26,
                    "Width": 365,
                    "Height": 16
                },
                "Polygon": [
                    {
                        "X": 33,
                        "Y": 26
                    },
                    {
                        "X": 398,
                        "Y": 25
                    },
                    {
                        "X": 399,
                        "Y": 41
                    },
                    {
                        "X": 34,
                        "Y": 43
                    }
                ],
                "Words": [

                ],
                "WordCoordPoint": [

                ],
                "AdvancedInfo": "{\"Parag\":{\"ParagNo\":2}}"
            },
            {
                "DetectedText": "How to convert std::vector<uint8 _t> to QByteArray? - Stack ...",
                "Confidence": 98,
                "ItemPolygon": {
                    "X": 34,
                    "Y": 51,
                    "Width": 547,
                    "Height": 21
                },
                "Polygon": [
                    {
                        "X": 34,
                        "Y": 51
                    },
                    {
                        "X": 581,
                        "Y": 51
                    },
                    {
                        "X": 581,
                        "Y": 72
                    },
                    {
                        "X": 34,
                        "Y": 72
                    }
                ],
                "Words": [

                ],
                "WordCoordPoint": [

                ],
                "AdvancedInfo": "{\"Parag\":{\"ParagNo\":3}}"
            }
        ],
        "Language": "zh",
        "Angel": 359.989990234375,
        "PdfPageSize": 0,
        "RequestId": "45854926-e728-4f64-9a54-eb39b826fa97"
    }
}
"#,
            )
            .is_ok()
        )
    }

    #[test]
    fn de_err() {
        assert!(
            serde_json::from_str::<HttpResponse<GeneralBasicOCRResponse>>(
                r#"
{
    "Response": {
        "Error": {
            "Code": "FailedOperation.ImageDecodeFailed",
            "Message": "图片解码失败"
        },
        "RequestId": "b6e509dd-8c48-434a-8edc-6a49f573d96c"
    }
}
"#,
            )
            .is_ok()
        )
    }
}
