use serde::{Deserialize, Serialize};

use super::api::HttpRequest;

#[derive(Serialize, Debug)]
#[allow(dead_code)]
pub enum OCRImage {
    #[serde(rename = "ImageBase64")]
    Base64(String),
    #[serde(rename = "ImageUrl")]
    Url(String),
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum OCRLanguageType {
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
pub struct GeneralBasicOCRRequest {
    #[serde(flatten)]
    image: OCRImage,
    #[serde(skip)]
    #[allow(dead_code)]
    scene: Option<String>,
    language_type: OCRLanguageType,
    is_pdf: bool,
    pdf_page_number: i32,
    is_words: bool,
}

impl HttpRequest for GeneralBasicOCRRequest {
    fn service() -> String {
        String::from("ocr")
    }

    fn host() -> String {
        String::from("ocr.tencentcloudapi.com")
    }

    fn action() -> String {
        String::from("GeneralBasicOCR")
    }

    fn version() -> String {
        String::from("2018-11-19")
    }
}

impl GeneralBasicOCRRequest {
    pub fn new(image: OCRImage, language_type: OCRLanguageType) -> GeneralBasicOCRRequest {
        GeneralBasicOCRRequest {
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
#[serde(rename_all = "PascalCase")]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ItemCoord {
    #[serde(flatten)]
    pub coord: Coord,
    pub width: i32,
    pub height: i32,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DetectedWords {
    pub confidence: i32,
    pub character: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DetectedWordCoordPoint {
    pub word_coordinate: (Coord, Coord, Coord, Coord),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct TextDetection {
    pub detected_text: String,
    pub confidence: i32,
    pub polygon: Vec<Coord>,
    pub advanced_info: String,
    pub item_polygon: ItemCoord,
    pub words: Vec<DetectedWords>,
    pub wrod_coord_point: Option<Vec<DetectedWordCoordPoint>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct GeneralBasicOCRResponse {
    pub text_detections: Vec<TextDetection>,
    pub language: String,
    pub angle: Option<f32>,
    pub pdf_page_size: i32,
    pub request_id: String,
}
