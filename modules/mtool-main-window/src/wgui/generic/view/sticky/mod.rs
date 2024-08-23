use mtool_wgui::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct TemplateView {
    pub id: String,
    pub template_id: TemplateId,
    pub template_data: TemplateData,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Command {
    ShowMain(TemplateView),
    ShowSub(TemplateView),
    LeftMouseUp,
}
