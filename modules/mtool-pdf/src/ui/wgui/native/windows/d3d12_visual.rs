use anyhow::Context;
use mtool_wgui::WGuiWindow;
use skia_safe as sk;
use std::sync::Arc;
use tauri::{PhysicalSize, WindowEvent};
use tokio::{
    sync::{oneshot, watch},
    task::LocalSet,
};
use tracing::{debug, warn};
use windows::{
    core::PCWSTR,
    Win32::{
        Graphics::{
            Direct3D::D3D_FEATURE_LEVEL_11_0,
            Direct3D12::{
                D3D12CreateDevice, D3D12GetDebugInterface, ID3D12CommandQueue, ID3D12Debug,
                ID3D12Debug1, ID3D12Device, ID3D12Resource, D3D12_COMMAND_LIST_TYPE_DIRECT,
                D3D12_COMMAND_QUEUE_DESC, D3D12_COMMAND_QUEUE_FLAG_NONE,
            },
            Dxgi::{
                Common::{DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM},
                CreateDXGIFactory1, IDXGIAdapter1, IDXGIFactory4, IDXGISwapChain1, IDXGISwapChain3,
                DXGI_ADAPTER_DESC1, DXGI_ADAPTER_FLAG, DXGI_ADAPTER_FLAG_SOFTWARE,
                DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_EFFECT_FLIP_DISCARD,
                DXGI_USAGE_RENDER_TARGET_OUTPUT,
            },
        },
        System::Threading::WaitForSingleObjectEx,
    },
};
use windows::{
    core::{ComInterface, PCSTR},
    Foundation::Numerics::Vector2,
    Win32::{
        Foundation::HANDLE,
        Graphics::Direct3D12::{ID3D12Fence, D3D12_FENCE_FLAG_NONE},
        System::{Threading::CreateEventA, WinRT::Composition::ICompositorInterop},
    },
    UI::Composition::{CompositionSurfaceBrush, Compositor, SpriteVisual},
};

use super::skia_context::SkiaContext;

pub struct RenderContext<'a> {
    pub frame_buffer_index: usize,
    pub canvas: &'a sk::Canvas,
}

pub type DrawHook = Box<dyn Fn(&mut RenderContext) -> Result<(), anyhow::Error> + Send>;
pub type PreResizeBuffersHook = Box<dyn Fn() -> Result<(), anyhow::Error> + Send>;
pub type PostResizeBuffersHook =
    Box<dyn Fn(&IDXGISwapChain1, PhysicalSize<u32>) -> Result<(), anyhow::Error> + Send>;

pub(super) struct D3d12Context {
    pub(super) size: PhysicalSize<u32>,
    pub(super) n_frames: u32,

    pub(super) adapter: IDXGIAdapter1,
    pub(super) device: ID3D12Device,

    pub(super) queue: ID3D12CommandQueue,
    pub(super) swap_chain: IDXGISwapChain1,

    fench: ID3D12Fence,
    fench_event: HANDLE,
    fench_values: Vec<u64>,
    frame_buffer_index: usize,
}

impl D3d12Context {
    pub fn new(size: PhysicalSize<u32>) -> Result<Self, anyhow::Error> {
        let PhysicalSize { width, height } = size;
        let n_frames = 2u32;

        Self::d3d12_debug_init()?;

        let (adapter, device, queue, swap_chain) = unsafe {
            let factory: IDXGIFactory4 = CreateDXGIFactory1()?;

            let (device, adapter) = Self::select_device(&factory)?;

            let queue = {
                let mut desc = D3D12_COMMAND_QUEUE_DESC::default();
                desc.Flags = D3D12_COMMAND_QUEUE_FLAG_NONE;
                desc.Type = D3D12_COMMAND_LIST_TYPE_DIRECT;
                device.CreateCommandQueue::<ID3D12CommandQueue>(&desc)?
            };

            let swap_chain = {
                let mut desc = DXGI_SWAP_CHAIN_DESC1::default();
                desc.BufferCount = 2;
                desc.Width = width;
                desc.Height = height;
                desc.Format = DXGI_FORMAT_B8G8R8A8_UNORM;
                desc.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
                desc.SwapEffect = DXGI_SWAP_EFFECT_FLIP_DISCARD;
                desc.SampleDesc.Count = 1;
                desc.AlphaMode = DXGI_ALPHA_MODE_PREMULTIPLIED;
                factory.CreateSwapChainForComposition(&queue, &desc, None)?
            };

            (adapter, device, queue, swap_chain)
        };

        let (fench, fench_event, fench_values) = unsafe {
            let fench_values = vec![0; n_frames as usize];

            let fench = device.CreateFence::<ID3D12Fence>(0, D3D12_FENCE_FLAG_NONE)?;
            let fench_event = CreateEventA(None, false, false, PCSTR::null())?;
            (fench, fench_event, fench_values)
        };

        let frame_buffer_index = Self::get_back_frame_buffer_index(&swap_chain)? as usize;

        Ok(Self {
            size,
            n_frames,

            adapter,
            device,
            queue,
            swap_chain,

            fench,
            fench_event,
            fench_values,
            frame_buffer_index,
        })
    }

    fn get_back_frame_buffer(&mut self) -> Result<(usize, ID3D12Resource), anyhow::Error> {
        let current_fench_value = self.fench_values[self.frame_buffer_index];

        self.frame_buffer_index = Self::get_back_frame_buffer_index(&self.swap_chain)? as usize;

        unsafe {
            if self.fench.GetCompletedValue() < self.fench_values[self.frame_buffer_index] {
                self.fench.SetEventOnCompletion(
                    self.fench_values[self.frame_buffer_index],
                    self.fench_event.clone(),
                )?;

                WaitForSingleObjectEx(self.fench_event, u32::max_value(), false);
            }
        }

        self.fench_values[self.frame_buffer_index] = current_fench_value + 1;

        Ok((self.frame_buffer_index, self.current_frame_buffer()?))
    }

    fn current_frame_buffer(&self) -> Result<ID3D12Resource, anyhow::Error> {
        self.get_frame_buffer(self.frame_buffer_index)
    }

    pub(super) fn get_frame_buffer(&self, i: usize) -> Result<ID3D12Resource, anyhow::Error> {
        Ok(unsafe { self.swap_chain.GetBuffer(i as u32)? })
    }

    fn swap_buffer(&mut self) -> Result<(), anyhow::Error> {
        unsafe {
            self.swap_chain.Present(1, 0).ok()?;
            self.queue
                .Signal(&self.fench, self.fench_values[self.frame_buffer_index])?;
        };
        Ok(())
    }

    fn d3d12_debug_init() -> Result<(), anyhow::Error> {
        #[cfg(debug_assertions)]
        {
            let mut debug: Option<ID3D12Debug> = None;

            unsafe {
                D3D12GetDebugInterface(&mut debug)?;
                if let Some(debug) = debug {
                    debug.EnableDebugLayer();
                    debug
                        .cast::<ID3D12Debug1>()?
                        .SetEnableGPUBasedValidation(true);
                }
            }
        }
        Ok(())
    }

    unsafe fn select_device(
        factory: &IDXGIFactory4,
    ) -> Result<(ID3D12Device, IDXGIAdapter1), anyhow::Error> {
        let mut device: Option<ID3D12Device> = None;
        let mut index = 0u32;
        while let Ok(adapter) = factory.EnumAdapters1(index) {
            let mut desc = DXGI_ADAPTER_DESC1::default();
            adapter.GetDesc1(&mut desc)?;
            debug!(
                "adapter description: {}",
                PCWSTR::from_raw(desc.Description.as_ptr()).display()
            );
            if DXGI_ADAPTER_FLAG(desc.Flags as i32).contains(DXGI_ADAPTER_FLAG_SOFTWARE) {
                index += 1;
                continue;
            }

            D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_11_0, &mut device)?;
            return Ok((device.context("create D3D12 device failed")?, adapter));
        }
        anyhow::bail!("Unable to find the right device!")
    }

    fn get_back_frame_buffer_index(swap_chain: &IDXGISwapChain1) -> Result<u32, anyhow::Error> {
        Ok(unsafe {
            swap_chain
                .cast::<IDXGISwapChain3>()?
                .GetCurrentBackBufferIndex()
        })
    }
}

pub struct D3d12Visual {
    win: Arc<WGuiWindow>,
    _compositor: Compositor,
    visual: SpriteVisual,

    d3d_context: D3d12Context,
    canvas_context: SkiaContext,

    draw_hook: Vec<DrawHook>,

    pre_resize_buffers_hook: Vec<PreResizeBuffersHook>,
    post_resize_buffers_hook: Vec<PostResizeBuffersHook>,
}

unsafe impl Send for D3d12Visual {}

impl D3d12Visual {
    pub fn new(
        win: Arc<WGuiWindow>,
        compositor: Compositor,
        size: PhysicalSize<u32>,
    ) -> Result<Self, anyhow::Error> {
        let context = D3d12Context::new(size)?;

        let visual = Self::create_visual(&compositor, &context.swap_chain, size)?;

        let canvas_context = SkiaContext::new(&context)?;

        Ok(Self {
            win,
            _compositor: compositor,
            visual,
            d3d_context: context,
            canvas_context,
            draw_hook: Vec::new(),
            pre_resize_buffers_hook: Vec::new(),
            post_resize_buffers_hook: Vec::new(),
        })
    }

    pub fn handle(&self) -> &SpriteVisual {
        return &self.visual;
    }

    pub fn set_draw_hooks(&mut self, hooks: Vec<DrawHook>) {
        self.draw_hook = hooks;
    }

    pub fn set_resize_buffers_hooks(
        &mut self,
        pre_hooks: Vec<PreResizeBuffersHook>,
        post_hook: Vec<PostResizeBuffersHook>,
    ) {
        self.pre_resize_buffers_hook = pre_hooks;
        self.post_resize_buffers_hook = post_hook;
    }

    async fn resize(&mut self, size: PhysicalSize<u32>) -> Result<(), anyhow::Error> {
        let PhysicalSize { width, height } = size;

        self.canvas_context.clear_surfaces()?;

        for hook in self.pre_resize_buffers_hook.iter() {
            (*hook)()?;
        }

        for i in 0..self.d3d_context.n_frames {
            unsafe {
                if self.d3d_context.fench.GetCompletedValue()
                    < self.d3d_context.fench_values[i as usize]
                {
                    self.d3d_context.fench.SetEventOnCompletion(
                        self.d3d_context.fench_values[i as usize],
                        self.d3d_context.fench_event.clone(),
                    )?;
                    WaitForSingleObjectEx(self.d3d_context.fench_event, u32::max_value(), false);
                }
            }
        }

        unsafe {
            self.d3d_context
                .swap_chain
                .ResizeBuffers(
                    self.d3d_context.n_frames,
                    width,
                    height,
                    DXGI_FORMAT_B8G8R8A8_UNORM,
                    0,
                )
                .context(format!("swapchain resize buffer: {:?}", size))?;
        }

        {
            let visual = self.visual.clone();
            let (tx, rx) = oneshot::channel();
            self.win.run_on_main_thread(move || {
                if let Err(e) = tx.send(visual.SetSize(Vector2 {
                    X: width as f32,
                    Y: height as f32,
                })) {
                    warn!("{:?}", e);
                }
            })?;
            rx.await??;
        }

        self.d3d_context.size = size;

        for hook in self.post_resize_buffers_hook.iter() {
            (*hook)(&self.d3d_context.swap_chain, size)?;
        }

        self.canvas_context.set_surfaces(&self.d3d_context)?;

        Ok(())
    }

    pub fn run(self) {
        let on_window_resize = {
            let (tx, rx) = watch::channel(self.d3d_context.size);
            self.win.on_window_event(move |e| match e {
                WindowEvent::Resized(size) => {
                    if size.width == 0 || size.height == 0 {
                        return;
                    }
                    if let Err(e) = tx.send(size.clone()) {
                        warn!("send resize event failed: {:?}", e);
                    }
                }
                _ => {}
            });

            rx
        };

        let _ = std::thread::Builder::new()
            .name("d3d12 renderer".to_string())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                let local = LocalSet::new();
                local.spawn_local(async move {
                    if let Err(e) = self.render_loop(on_window_resize).await {
                        warn!("{:?}", e);
                    }
                });
                rt.block_on(local);
            });
    }

    async fn render_loop(
        mut self,
        mut on_window_resize: watch::Receiver<PhysicalSize<u32>>,
    ) -> Result<(), anyhow::Error> {
        loop {
            if let Some(size) = {
                let size = on_window_resize.borrow_and_update();
                if size.has_changed() {
                    Some(*size)
                } else {
                    None
                }
            } {
                self.resize(size).await?;
            }

            let (frame_buffer_index, _frame_buffer) = self.d3d_context.get_back_frame_buffer()?;

            let mut surface = self.canvas_context.get_surface(frame_buffer_index);
            let canvas = surface.canvas();

            canvas.clear(sk::Color::WHITE);

            let mut ctx = RenderContext {
                frame_buffer_index,
                canvas,
            };
            for hook in self.draw_hook.iter() {
                hook(&mut ctx)?;
            }

            self.canvas_context.flush_and_submit(&mut surface);

            self.d3d_context.swap_buffer()?;
        }
    }

    fn create_brush(
        compositor: &Compositor,
        swap_chain: &IDXGISwapChain1,
    ) -> Result<CompositionSurfaceBrush, anyhow::Error> {
        let surface = unsafe {
            compositor
                .cast::<ICompositorInterop>()?
                .CreateCompositionSurfaceForSwapChain(swap_chain)?
        };

        Ok(compositor.CreateSurfaceBrushWithSurface(&surface)?)
    }

    fn create_visual(
        compositor: &Compositor,
        swap_chain: &IDXGISwapChain1,
        PhysicalSize { width, height }: PhysicalSize<u32>,
    ) -> Result<SpriteVisual, anyhow::Error> {
        let visual = compositor.CreateSpriteVisual()?;

        visual.SetBrush(&Self::create_brush(compositor, swap_chain)?)?;
        visual.SetSize(Vector2 {
            X: width as f32,
            Y: height as f32,
        })?;

        Ok(visual)
    }
}
