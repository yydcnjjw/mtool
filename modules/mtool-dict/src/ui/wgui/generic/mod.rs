use mtool_wgui::{TemplateData, TemplateId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub template_id: TemplateId,
    pub data: TemplateData,
}
