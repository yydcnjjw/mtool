use std::ops::{Deref, DerefMut};

use mcloud_api::adobe;
use sea_orm::{entity::prelude::*, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mtool_pdf_adobe")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Vec<u8>,

    pub structure: PdfStructure,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct PdfStructure {
    #[serde(flatten)]
    inner: adobe::PdfStructure,
}

impl PdfStructure {
    pub fn new(inner: adobe::PdfStructure) -> Self {
        Self { inner }
    }
}

impl Deref for PdfStructure {
    type Target = adobe::PdfStructure;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for PdfStructure {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl From<PdfStructure> for adobe::PdfStructure {
    fn from(value: PdfStructure) -> Self {
        value.inner
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
