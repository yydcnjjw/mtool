use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfStructure {
    pub version: Version,
    pub extended_metadata: ExtendedMetadata,
    pub elements: Vec<Element>,
    pub pages: Vec<Page>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Version {
    pub json_export: String,
    pub page_segmentation: String,
    pub schema: String,
    pub structure: String,
    pub table_structure: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtendedMetadata {
    #[serde(rename = "ID_instance")]
    pub id_instance: String,
    #[serde(rename = "ID_permanent")]
    pub id_permanent: String,
    pub pdf_version: String,
    pub pdfa_compliance_level: String,
    pub is_encrypted: bool,
    pub has_acroform: bool,
    pub is_digitally_signed: bool,
    pub pdfua_compliance_level: String,
    pub page_count: i64,
    pub has_embedded_files: bool,
    pub is_certified: bool,
    #[serde(rename = "is_XFA")]
    pub is_xfa: bool,
    pub language: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Element {
    #[serde(rename = "Bounds")]
    pub bounds: Option<[f64; 4]>,
    #[serde(rename = "Font")]
    pub font: Option<Font>,
    #[serde(rename = "HasClip")]
    pub has_clip: Option<bool>,
    #[serde(rename = "Lang")]
    pub lang: Option<String>,
    #[serde(rename = "Page")]
    pub page: Option<u16>,
    #[serde(rename = "Path")]
    pub path: String,
    #[serde(rename = "Text")]
    pub text: Option<String>,
    #[serde(rename = "TextSize")]
    pub text_size: Option<f64>,
    pub attributes: Option<Attributes>,
    #[serde(rename = "elementId")]
    pub element_id: Option<i64>,
    #[serde(rename = "filePaths")]
    #[serde(default)]
    pub file_paths: Vec<String>,
    #[serde(rename = "Rotation")]
    pub rotation: Option<f32>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Font {
    pub alt_family_name: String,
    pub embedded: bool,
    pub encoding: String,
    pub family_name: String,
    pub font_type: String,
    pub italic: bool,
    pub monospaced: bool,
    pub name: String,
    pub subset: bool,
    pub weight: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attributes {
    #[serde(rename = "LineHeight")]
    pub line_height: Option<f64>,
    #[serde(rename = "TextAlign")]
    pub text_align: Option<String>,
    #[serde(rename = "BBox")]
    pub bbox: Option<Vec<f64>>,
    #[serde(rename = "BlockAlign")]
    pub block_align: Option<String>,
    #[serde(rename = "BorderColor")]
    pub border_color: Option<Vec<Option<Vec<f64>>>>,
    #[serde(rename = "BorderStyle")]
    pub border_style: Option<Vec<String>>,
    #[serde(rename = "BorderThickness")]
    pub border_thickness: Option<Vec<f64>>,
    #[serde(rename = "ColIndex")]
    pub col_index: Option<i64>,
    #[serde(rename = "Height")]
    pub height: Option<f64>,
    #[serde(rename = "InlineAlign")]
    pub inline_align: Option<String>,
    #[serde(rename = "RowIndex")]
    pub row_index: Option<i64>,
    #[serde(rename = "Width")]
    pub width: Option<f64>,
    #[serde(rename = "Placement")]
    pub placement: Option<String>,
    #[serde(rename = "TextPosition")]
    pub text_position: Option<String>,
    #[serde(rename = "BaselineShift")]
    pub baseline_shift: Option<f64>,
    #[serde(rename = "NumCol")]
    pub num_col: Option<i64>,
    #[serde(rename = "NumRow")]
    pub num_row: Option<i64>,
    #[serde(rename = "SpaceAfter")]
    pub space_after: Option<f64>,
    #[serde(rename = "ColSpan")]
    pub col_span: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page {
    pub boxes: Boxes,
    pub height: f32,
    pub is_scanned: bool,
    pub page_number: i64,
    pub rotation: f32,
    pub width: f32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Boxes {
    #[serde(rename = "CropBox")]
    pub crop_box: Vec<f32>,
    #[serde(rename = "MediaBox")]
    pub media_box: Vec<f32>,
}
