use anyhow::Context;
use skia_safe::{
    gpu::{self, d3d, DirectContext},
    Surface,
};
use tauri::PhysicalSize;
use windows::{
    core::Interface,
    Win32::Graphics::{
        Direct3D12::D3D12_RESOURCE_STATE_PRESENT,
        Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
    },
};
use wio::com::ComPtr;

use super::d3d12_visual::D3d12Context;

fn new_com_ptr<O: winapi::Interface, T: Interface>(v: T) -> ComPtr<O> {
    unsafe { ComPtr::from_raw(v.into_raw().cast::<O>()) }
}

pub struct SkiaContext {
    context: DirectContext,
    surfaces: Vec<Surface>,
}

impl SkiaContext {
    pub(super) fn new(d3d_context: &D3d12Context) -> Result<Self, anyhow::Error> {
        let mut context = unsafe {
            DirectContext::new_d3d(
                &d3d::BackendContext {
                    adapter: new_com_ptr(d3d_context.adapter.clone()),
                    device: new_com_ptr(d3d_context.device.clone()),
                    queue: new_com_ptr(d3d_context.queue.clone()),
                    memory_allocator: None,
                    protected_context: gpu::Protected::No,
                },
                None,
            )
        }
        .context("create direct context failed")?;

        let surfaces = Self::create_surfaces(d3d_context, &mut context)?;

        Ok(Self { context, surfaces })
    }

    pub(super) fn get_surface(&self, index: usize) -> Surface {
        self.surfaces[index].clone()
    }

    pub(super) fn flush_and_submit(&mut self, _surface: &mut Surface) {
        self.context.flush_and_submit();
    }

    fn create_surfaces(
        d3d_context: &D3d12Context,
        context: &mut DirectContext,
    ) -> Result<Vec<Surface>, anyhow::Error> {
        let PhysicalSize { width, height } = d3d_context.size;

        let mut surfaces = Vec::new();

        for i in 0..d3d_context.n_frames {
            let buffer = d3d_context.get_frame_buffer(i as usize)?;

            let info = d3d::TextureResourceInfo {
                resource: new_com_ptr(buffer.clone()),
                alloc: None,
                resource_state: D3D12_RESOURCE_STATE_PRESENT.0 as u32,
                format: DXGI_FORMAT_B8G8R8A8_UNORM.0 as u32,
                sample_count: 1,
                level_count: 1,
                sample_quality_pattern: 0,
                protected: gpu::Protected::No,
            };

            let rt = gpu::BackendRenderTarget::new_d3d((width as i32, height as i32), &info);

            let surface = gpu::surfaces::wrap_backend_render_target(
                context,
                &rt,
                gpu::SurfaceOrigin::TopLeft,
                skia_safe::ColorType::BGRA8888,
                None,
                None,
            )
            .context("create skia surface failed")?;

            surfaces.push(surface);
        }
        Ok(surfaces)
    }

    pub(super) fn clear_surfaces(&mut self) -> Result<(), anyhow::Error> {
        self.context.flush_submit_and_sync_cpu();
        self.surfaces.clear();
        Ok(())
    }

    pub(super) fn set_surfaces(&mut self, context: &D3d12Context) -> Result<(), anyhow::Error> {
        self.surfaces = Self::create_surfaces(context, &mut self.context)?;
        Ok(())
    }
}
