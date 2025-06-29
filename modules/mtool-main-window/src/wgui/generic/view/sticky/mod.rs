use mapp::serde::{Deserialize, Serialize};
use mtool_wgui::prelude::*;

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "mapp::serde")]
pub struct TemplateView {
    pub id: String,
    pub template_id: TemplateId,
    pub template_data: TemplateData,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "mapp::serde")]
pub enum Command {
    ShowMain(TemplateView),
    LeftMouseUp,
}
