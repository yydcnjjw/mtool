use async_trait::async_trait;
use mapp::prelude::*;
use mtool_core::{CmdlineStage, ConfigStore};
use pdfium_render::prelude::*;
use std::{ops::Deref, sync::OnceLock};

use crate::Config;

static PDF_INST: OnceLock<Pdf> = OnceLock::new();

pub struct Pdf {
    inner: Pdfium,
}

impl Deref for Pdf {
    type Target = Pdfium;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Pdf {
    async fn init(cs: Res<ConfigStore>) -> Result<(), anyhow::Error> {
        PDF_INST
            .set(Self::new(&cs.get("pdf").await?)?)
            .map_err(|_| anyhow::anyhow!("initialize Pdf instance failed"))
    }

    fn new(config: &Config) -> Result<Self, anyhow::Error> {
        let bindings = Pdfium::bind_to_library(&config.library)
            .map_err(|e| anyhow::anyhow!("Failed to load pdfium library: {}", e))?;

        Ok(Self {
            inner: Pdfium::new(bindings),
        })
    }

    #[allow(unused)]
    pub fn get() -> Option<&'static Pdf> {
        PDF_INST.get()
    }

    pub fn get_unwrap() -> &'static Pdf {
        PDF_INST.get().unwrap()
    }    
}

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.schedule()
            .add_once_task(CmdlineStage::AfterInit, Pdf::init);
        Ok(())
    }
}
