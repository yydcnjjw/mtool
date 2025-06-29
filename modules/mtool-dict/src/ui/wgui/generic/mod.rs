use mapp::serde::{Deserialize, Serialize};
use mtool_wgui::{TemplateData, TemplateId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct QueryResult {
    pub template_id: TemplateId,
    pub data: TemplateData,
}
