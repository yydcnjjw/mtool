use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::PdfStructure;

#[derive(EnumIter, DeriveActiveEnum, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum State {
    GetAssetId = 0,
    Upload = 1,
    ExtractPdf = 2,
    End = 3,
}

#[derive(DeriveEntityModel, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[sea_orm(table_name = "mtool_pdf_adobe")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Vec<u8>,
    pub media_type: String,
    
    pub asset_id: Option<String>,
    pub upload_uri: Option<String>,

    pub structure: Option<PdfStructure>,

    pub state: State,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
