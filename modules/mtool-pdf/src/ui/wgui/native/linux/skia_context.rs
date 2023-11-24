use std::ffi::CString;

use anyhow::Context;
use sk::gpu::{self, SurfaceOrigin};
use sk::gpu::{gl::FramebufferInfo, DirectContext};
use skia_safe as sk;

use gl::types::*;
use glutin::config::Config as GLConfig;
use glutin::display::Display;
use glutin::prelude::*;
use tauri::PhysicalSize;

pub struct SkiaContext {
    gr_context: DirectContext,
    surface: sk::Surface,
    fb_info: FramebufferInfo,

    num_samples: usize,
    stencil_size: usize,
}

impl SkiaContext {
    pub(super) fn new_with_gl(
        size: PhysicalSize<u32>,
        display: &Display,
        gl_config: &GLConfig,
    ) -> Result<Self, anyhow::Error> {
        let fb_info = {
            let mut fboid: GLint = 0;
            unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };

            FramebufferInfo {
                fboid: fboid.try_into()?,
                format: skia_safe::gpu::gl::Format::RGBA8.into(),
                ..Default::default()
            }
        };

        let interface = skia_safe::gpu::gl::Interface::new_load_with(|name| {
            if name == "eglGetCurrentDisplay" {
                return std::ptr::null();
            }
            display.get_proc_address(CString::new(name).unwrap().as_c_str())
        })
        .context("create GL interface")?;

        Self::new(
            interface,
            size,
            fb_info,
            gl_config.num_samples() as usize,
            gl_config.stencil_size() as usize,
        )
    }

    fn new(
        interface: skia_safe::gpu::gl::Interface,
        size: PhysicalSize<u32>,
        fb_info: FramebufferInfo,
        num_samples: usize,
        stencil_size: usize,
    ) -> Result<Self, anyhow::Error> {
        let mut gr_context = skia_safe::gpu::DirectContext::new_gl(Some(interface), None)
            .context("create gr context failed")?;
        let surface =
            Self::create_surface(size, fb_info, &mut gr_context, num_samples, stencil_size)?;
        Ok(Self {
            gr_context,
            surface,
            fb_info,
            num_samples,
            stencil_size,
        })
    }

    fn create_surface(
        PhysicalSize { width, height }: PhysicalSize<u32>,
        fb_info: FramebufferInfo,
        gr_context: &mut skia_safe::gpu::DirectContext,
        num_samples: usize,
        stencil_size: usize,
    ) -> Result<sk::Surface, anyhow::Error> {
        let backend_render_target = gpu::backend_render_targets::make_gl(
            (width as i32, height as i32),
            num_samples,
            stencil_size,
            fb_info,
        );

        gpu::surfaces::wrap_backend_render_target(
            gr_context,
            &backend_render_target,
            SurfaceOrigin::BottomLeft,
            sk::ColorType::RGBA8888,
            None,
            None,
        )
        .context("create surface")
    }

    pub(super) fn recreate_surface(
        &mut self,
        size: PhysicalSize<u32>,
    ) -> Result<(), anyhow::Error> {
        self.surface = Self::create_surface(
            size,
            self.fb_info,
            &mut self.gr_context,
            self.num_samples,
            self.stencil_size,
        )?;
        Ok(())
    }

    pub(super) fn canvas(&mut self) -> &sk::Canvas {
        self.surface.canvas()
    }

    pub(super) fn flush_and_submit(&mut self) {
        self.gr_context.flush_and_submit();
    }
}
