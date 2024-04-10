use async_trait::async_trait;
use mapp::prelude::*;
use mtool_core::ConfigStore;
use pdfium_render::prelude::*;
use std::ops::Deref;
use tokio::sync::OnceCell;

use crate::Config;

static PDFIUM: OnceCell<Pdfium> = OnceCell::const_new();

pub struct PdfApi {
    inner: &'static Pdfium,
}

impl Deref for PdfApi {
    type Target = Pdfium;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl PdfApi {
    async fn construct(cs: Res<ConfigStore>) -> Result<Res<Self>, anyhow::Error> {
        Ok(Res::new(Self::new(&cs.get("pdf").await?)?))
    }

    fn new(config: &Config) -> Result<Self, anyhow::Error> {
        let bindings = Pdfium::bind_to_library(&config.pdfium)
            .map_err(|e| anyhow::anyhow!("Failed to load pdfium library: {}", e))?;

        PDFIUM.set(Pdfium::new(bindings))?;

        Ok(Self {
            inner: PDFIUM.get().unwrap(),
        })
    }

    pub fn get(&self) -> &'static Pdfium {
        self.inner
    }
}

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, ctx: &mut AppContext) -> Result<(), anyhow::Error> {
        ctx.injector().construct_once(PdfApi::construct);
        Ok(())
    }
}
