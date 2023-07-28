use pdfium_render::prelude::PdfiumError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Pdf(#[from] PdfiumError),
    // #[error(transparent)]
    // Draw(#[from] piet_common::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
