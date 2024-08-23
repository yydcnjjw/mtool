use std::{iter, ptr::{self, null}};

use anyhow::Context;
use tauri::PhysicalSize;
use tracing::debug;
use wgpu::{hal, SurfaceTargetUnsafe};
use windows::{
    core::{Interface, PCSTR, PCWSTR},
    Win32::{
        Foundation::HANDLE,
        Graphics::{
            Direct3D::D3D_FEATURE_LEVEL_11_0,
            Direct3D12::{
                D3D12CreateDevice, D3D12GetDebugInterface, ID3D12CommandQueue, ID3D12Debug,
                ID3D12Debug1, ID3D12Device, ID3D12Fence, ID3D12Resource,
                D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_COMMAND_QUEUE_DESC,
                D3D12_COMMAND_QUEUE_FLAG_NONE, D3D12_FENCE_FLAG_NONE,
            },
            Dxgi::{
                Common::{DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM},
                CreateDXGIFactory1, IDXGIAdapter1, IDXGIFactory4, IDXGISwapChain1, IDXGISwapChain3,
                DXGI_ADAPTER_DESC1, DXGI_ADAPTER_FLAG, DXGI_ADAPTER_FLAG_SOFTWARE, DXGI_PRESENT,
                DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_EFFECT_FLIP_DISCARD,
                DXGI_USAGE_RENDER_TARGET_OUTPUT,
            },
        },
        System::Threading::{CreateEventA, WaitForSingleObjectEx},
    },
};

pub(super) struct WGPUContext {
    pub adapter: wgpu::Adapter,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
}

impl WGPUContext {
    pub async fn new(size: PhysicalSize<u32>) -> Result<Self, anyhow::Error> {
        let PhysicalSize { width, height } = size;

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = unsafe {
            instance.create_surface_unsafe(SurfaceTargetUnsafe::CompositionVisual(ptr::null_mut()))
        }?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .context("request adapter failed")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await?;

        let caps = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Ok(Self {
            adapter,
            surface,
            device,
            queue,
            config,
            size,
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }


    fn render(&mut self) -> Result<(), anyhow::Error> {
        if self.size.width == 0 || self.size.height == 0 {
            return Ok(());
        }
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        unsafe {
            view.as_hal::<hal::api::Dx12>(|view|{
                view.unwrap()
            });
        }

        // let mut encoder = self
        //     .device
        //     .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        //         label: Some("Render Encoder"),
        //     });

        // {
        //     let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //         label: Some("Render Pass"),
        //         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        //             view: &view,
        //             resolve_target: None,
        //             ops: wgpu::Operations {
        //                 load: wgpu::LoadOp::Clear(wgpu::Color {
        //                     r: 0.1,
        //                     g: 0.2,
        //                     b: 0.3,
        //                     a: 1.0,
        //                 }),
        //                 store: wgpu::StoreOp::Store,
        //             },
        //         })],
        //         ..Default::default()
        //     });
        // }

        // self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
        

    // fn get_back_frame_buffer(&mut self) -> Result<(usize, ID3D12Resource), anyhow::Error> {
    //     let current_fench_value = self.fench_values[self.frame_buffer_index];

    //     self.frame_buffer_index = Self::get_back_frame_buffer_index(&self.swap_chain)? as usize;

    //     unsafe {
    //         if self.fench.GetCompletedValue() < self.fench_values[self.frame_buffer_index] {
    //             self.fench.SetEventOnCompletion(
    //                 self.fench_values[self.frame_buffer_index],
    //                 self.fench_event.clone(),
    //             )?;

    //             WaitForSingleObjectEx(self.fench_event, u32::max_value(), false);
    //         }
    //     }

    //     self.fench_values[self.frame_buffer_index] = current_fench_value + 1;

    //     Ok((self.frame_buffer_index, self.current_frame_buffer()?))
    // }

    // fn current_frame_buffer(&self) -> Result<ID3D12Resource, anyhow::Error> {
    //     self.get_frame_buffer(self.frame_buffer_index)
    // }

    // pub(super) fn get_frame_buffer(&self, i: usize) -> Result<ID3D12Resource, anyhow::Error> {
    //     Ok(unsafe { self.swap_chain.GetBuffer(i as u32)? })
    // }

    // fn swap_buffer(&mut self) -> Result<(), anyhow::Error> {
    //     unsafe {
    //         self.swap_chain.Present(1, DXGI_PRESENT(0)).ok()?;
    //         self.queue
    //             .Signal(&self.fench, self.fench_values[self.frame_buffer_index])?;
    //     };
    //     Ok(())
    // }

    // fn d3d12_debug_init() -> Result<(), anyhow::Error> {
    //     #[cfg(debug_assertions)]
    //     {
    //         let mut debug: Option<ID3D12Debug> = None;

    //         unsafe {
    //             D3D12GetDebugInterface(&mut debug)?;
    //             if let Some(debug) = debug {
    //                 debug.EnableDebugLayer();
    //                 debug
    //                     .cast::<ID3D12Debug1>()?
    //                     .SetEnableGPUBasedValidation(true);
    //             }
    //         }
    //     }
    //     Ok(())
    // }

    // unsafe fn select_device(
    //     factory: &IDXGIFactory4,
    // ) -> Result<(ID3D12Device, IDXGIAdapter1), anyhow::Error> {
    //     let mut device: Option<ID3D12Device> = None;
    //     let mut index = 0u32;
    //     while let Ok(adapter) = factory.EnumAdapters1(index) {
    //         let desc = DXGI_ADAPTER_DESC1::default();
    //         adapter.GetDesc1()?;
    //         debug!(
    //             "adapter description: {}",
    //             PCWSTR::from_raw(desc.Description.as_ptr()).display()
    //         );
    //         if DXGI_ADAPTER_FLAG(desc.Flags as i32).contains(DXGI_ADAPTER_FLAG_SOFTWARE) {
    //             index += 1;
    //             continue;
    //         }

    //         D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_11_0, &mut device)?;
    //         return Ok((device.context("create D3D12 device failed")?, adapter));
    //     }
    //     anyhow::bail!("Unable to find the right device!")
    // }

    // fn get_back_frame_buffer_index(swap_chain: &IDXGISwapChain1) -> Result<u32, anyhow::Error> {
    //     Ok(unsafe {
    //         swap_chain
    //             .cast::<IDXGISwapChain3>()?
    //             .GetCurrentBackBufferIndex()
    //     })
    // }
}
