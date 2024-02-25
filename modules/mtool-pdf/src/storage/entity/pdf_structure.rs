use mcloud_api::adobe;
use sea_orm::{entity::prelude::*, FromJsonQueryResult};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

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
