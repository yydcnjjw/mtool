use anyhow::Context;
use glutin::config::Config as GLConfig;
use glutin::prelude::*;
use glutin::{
    config::ConfigTemplateBuilder,
    display::{Display, DisplayApiPreference},
};
use gtk::gdk::GLContext;
use gtk::glib::Propagation;
use gtk::prelude::*;
use gtk::Overlay;
use gtk::{gdk::GLError, GLArea};
use mtool_wgui::WGuiWindow;
use raw_window_handle::HasRawDisplayHandle;
use skia_safe as sk;
use std::ffi::CString;
use std::sync::Arc;
use tauri::PhysicalSize;
use tracing::warn;

use super::skia_context::SkiaContext;

type DrawHook = Box<dyn Fn(&mut RenderContext) -> Result<(), anyhow::Error> + Send>;

pub struct RendererBuilder {
    win: Arc<WGuiWindow>,

    draw_hook: Vec<DrawHook>,
}

impl RendererBuilder {
    pub fn new(win: Arc<WGuiWindow>) -> Self {
        Self {
            win,
            draw_hook: Vec::new(),
        }
    }

    pub fn add_draw_hook<Hook>(mut self, hook: Hook) -> Self
    where
        Hook: Fn(&mut RenderContext) -> Result<(), anyhow::Error> + Send + 'static,
    {
        self.draw_hook.push(Box::new(hook));
        self
    }

    pub async fn build(self) -> Result<Renderer, anyhow::Error> {
        Renderer::new(self).await
    }
}

pub struct RenderContext<'a> {
    pub canvas: &'a sk::Canvas,
}

pub struct Renderer {
    _win: Arc<WGuiWindow>,
}

impl Renderer {
    fn find_config(display: &Display) -> Result<GLConfig, anyhow::Error> {
        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_transparency(true)
            .build();

        unsafe { display.find_configs(template)? }
            .reduce(|accum, config| {
                let transparency_check = config.supports_transparency().unwrap_or(false)
                    & !accum.supports_transparency().unwrap_or(false);

                if transparency_check || config.num_samples() < accum.num_samples() {
                    config
                } else {
                    accum
                }
            })
            .context("try find GL config")
    }

    fn rebuild_widget(win: Arc<WGuiWindow>) -> Result<GLArea, anyhow::Error> {
        let win = win.gtk_window()?;
        let widget = win.children().pop().context("root widget is not exist")?;
        let vbox = widget
            .downcast::<gtk::Box>()
            .map_err(|e| anyhow::anyhow!("downcast to gtk::Box failed: {}", e))?;
        let webview = vbox.children().pop().context("not wry window")?;
        {
            vbox.remove(&webview);
            win.remove(&vbox);
        }

        let glarea = GLArea::new();
        let overlay = Overlay::new();
        overlay.add_overlay(&glarea);
        overlay.add_overlay(&webview);
        win.add(&overlay);

        Ok(glarea)
    }

    const SKIA_CONTEXT_KEY: &'static str = "SKIA_CONTEXT";

    fn on_realize(
        glarea: &GLArea,
        display: &Display,
        gl_config: &GLConfig,
    ) -> Result<(), anyhow::Error> {
        glarea.make_current();

        if glarea.error().is_some() {
            return Ok(());
        }

        let size = glarea.allocation();
        let skia_context = SkiaContext::new_with_gl(
            PhysicalSize::new(size.width() as u32, size.height() as u32),
            display,
            gl_config,
        )?;

        unsafe {
            glarea.set_data(Self::SKIA_CONTEXT_KEY, skia_context);
        }

        Ok(())
    }

    fn on_resize(glarea: &GLArea, width: i32, height: i32) -> Result<(), anyhow::Error> {
        if let Some(mut skia_context) =
            unsafe { glarea.data::<SkiaContext>(Self::SKIA_CONTEXT_KEY) }
        {
            unsafe { skia_context.as_mut() }
                .recreate_surface(PhysicalSize::new(width as u32, height as u32))?;
        }
        Ok(())
    }

    fn on_render(
        glarea: &GLArea,
        _gl_context: &GLContext,
        draw_hooks: &Vec<DrawHook>,
    ) -> Result<(), anyhow::Error> {
        if let Some(mut skctx) = unsafe { glarea.data::<SkiaContext>(Self::SKIA_CONTEXT_KEY) } {
            let skctx = unsafe { skctx.as_mut() };
            let canvas = skctx.canvas();

            let mut rc = RenderContext { canvas };
            for hook in draw_hooks {
                hook(&mut rc)?;
            }
            skctx.flush_and_submit();
        }
        Ok(())
    }

    fn setup(builder: RendererBuilder) -> Result<(), anyhow::Error> {
        let RendererBuilder { win, draw_hook } = builder;

        let display = unsafe { Display::new(win.raw_display_handle(), DisplayApiPreference::Egl)? };

        gl::load_with(|s| display.get_proc_address(CString::new(s).unwrap().as_c_str()));

        let gl_config = Self::find_config(&display)?;

        let glarea = Self::rebuild_widget(win)?;

        glarea.connect_realize(move |glarea| {
            if let Err(e) = Self::on_realize(glarea, &display, &gl_config) {
                return glarea.set_error(Some(&gtk::glib::Error::new(
                    GLError::__Unknown(-1),
                    &e.to_string(),
                )));
            }
        });

        glarea.connect_resize(move |glarea, width, height| {
            if let Err(e) = Self::on_resize(glarea, width, height) {
                return glarea.set_error(Some(&gtk::glib::Error::new(
                    GLError::__Unknown(-1),
                    &e.to_string(),
                )));
            }
        });

        glarea.connect_render(move |glarea, context| {
            if let Err(e) = Self::on_render(glarea, context, &draw_hook) {
                glarea.set_error(Some(&gtk::glib::Error::new(
                    GLError::__Unknown(-1),
                    &e.to_string(),
                )));
            }
            Propagation::Proceed
        });

        Ok(())
    }

    pub async fn new(builder: RendererBuilder) -> Result<Self, anyhow::Error> {
        let win = builder.win.clone();

        win.run_on_main_thread(move || {
            if let Err(e) = Self::setup(builder) {
                warn!("{:?}", e);
            }
        })?;

        Ok(Self { _win: win })
    }
}
