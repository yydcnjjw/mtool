use std::fmt::{format, Debug};

use chrono::prelude::*;

use crypto::{digest::Digest, hmac::Hmac, mac::Mac, sha2::Sha256};
use reqwest::header::{CONTENT_TYPE, HOST};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
enum OCRImage {
    #[serde(rename = "ImageBase64")]
    Base64(String),
    #[serde(rename = "ImageUrl")]
    Url(String),
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum OCRLanguageType {
    Zh,
    Auto,
    Jap,
    Kor,
    Spa,
    Fre,
    Ger,
    Por,
    Vie,
    May,
    Rus,
    Ita,
    Hol,
    Swe,
    Fin,
    Dan,
    Nor,
    Hun,
    Tha,
    Lat,
    Ara,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct GeneralBasicOCRRequest {
    #[serde(skip)]
    action: &'static str,
    #[serde(skip)]
    version: &'static str,
    #[serde(skip)]
    region: String,
    #[serde(flatten)]
    image: OCRImage,
    #[serde(skip)]
    scene: Option<String>,
    language_type: OCRLanguageType,
    is_pdf: bool,
    pdf_page_number: i32,
    is_words: bool,
}

impl GeneralBasicOCRRequest {
    fn new(image: OCRImage, language_type: OCRLanguageType) -> GeneralBasicOCRRequest {
        GeneralBasicOCRRequest {
            action: "GeneralBasicOCR",
            version: "2018-11-19",
            region: "ap-shanghai".to_string(),
            image,
            language_type,
            is_pdf: false,
            scene: Option::None,
            pdf_page_number: 1,
            is_words: false,
        }
    }
}

#[derive(Deserialize, Debug)]
struct Coord {
    #[serde(rename = "X")]
    x: i32,
    #[serde(rename = "Y")]
    y: i32,
}

#[derive(Deserialize, Debug)]
struct ItemCoord {
    #[serde(flatten)]
    coord: Coord,
    #[serde(rename = "Width")]
    width: i32,
    #[serde(rename = "Height")]
    height: i32,
}
#[derive(Deserialize, Debug)]
struct DetectedWords {
    #[serde(rename = "Confidence")]
    confidence: i32,
    #[serde(rename = "Character")]
    character: String,
}

#[derive(Deserialize, Debug)]
struct DetectedWordCoordPoint {
    #[serde(rename = "WordCoordinate")]
    word_coordinate: (Coord, Coord, Coord, Coord),
}

#[derive(Deserialize, Debug)]
struct TextDetection {
    #[serde(rename = "DetectedText")]
    detected_text: String,
    #[serde(rename = "Confidence")]
    confidence: i32,
    #[serde(rename = "Polygon")]
    polygon: Vec<Coord>,
    #[serde(rename = "AdvancedInfo")]
    advanced_info: String,
    #[serde(rename = "ItemPolygon")]
    item_polygon: ItemCoord,
    #[serde(rename = "Words")]
    words: Vec<DetectedWords>,
    #[serde(rename = "WordCoordPoint")]
    wrod_coord_point: Vec<DetectedWordCoordPoint>,
}
#[derive(Deserialize, Debug)]
struct Response<T> {
    #[serde(rename = "Response")]
    response: T
}

#[derive(Deserialize, Debug)]
struct GeneralBasicOCRResponse {
    #[serde(rename = "TextDetections")]
    text_detections: Vec<TextDetection>,
    #[serde(rename = "Language")]
    language: String,
    #[serde(rename = "Angel")]
    angle: f32,
    #[serde(rename = "PdfPageSize")]
    pdf_page_size: i32,
    #[serde(rename = "RequestId")]
    request_id: String,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let utc: DateTime<Utc> = Utc::now();
    let secret_id = "AKIDxnU3TlI54iG0b3wfLjw1ylV7bxmVHYyF";
    let secret_key = "GI4Wu69LHxOSJvMYVH1bK0b4nrrlX4LS";

    let service = "ocr";
    let host = "ocr.tencentcloudapi.com";
    let endpoint = format!("https://{}", host);

    let action = "GeneralBasicOCR";
    let version = "2018-11-19";

    let algorithm = "TC3-HMAC-SHA256";
    let timestamp = utc.timestamp();
    let date = utc.format("%Y-%m-%d").to_string();

    let method = "POST";
    let canonical_uri = "/";
    let canonical_querystring = "";
    let ct = "application/json; charset=utf-8";

    let payload = serde_json::to_string(&GeneralBasicOCRRequest::new(
        OCRImage::Base64(base64::encode(include_bytes!("/tmp/test.png"))),
        OCRLanguageType::Zh,
    ))
    .unwrap();

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

    let resp = reqwest::Client::new()
        .post(endpoint)
        .header(HOST, host)
        .header(CONTENT_TYPE, ct)
        .header("X-TC-Action", action)
        .header("X-TC-Version", version)
        .header("X-TC-Timestamp", timestamp)
        .header("X-TC-Region", "ap-shanghai")
        .header("Authorization", authorization)
        .body(payload)
        .send()
        .await?
        .json::<Response<GeneralBasicOCRResponse>>()
        .await?;

    println!("{:?}", resp);
    Ok(())
}

pub fn run() {
    println!("ocr");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
