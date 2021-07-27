use serde::{Deserialize, Serialize};

use super::api::HttpRequest;

#[derive(Serialize, Debug)]
pub enum OCRImage {
    #[serde(rename = "ImageBase64")]
    Base64(String),
    #[serde(rename = "ImageUrl")]
    Url(String),
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
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
pub struct GeneralBasicOCRResponse {
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
