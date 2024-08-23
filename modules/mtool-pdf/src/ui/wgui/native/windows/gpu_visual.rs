use mtool_wgui::WGuiWindow;
use skia_safe as sk;
use std::sync::Arc;
use tauri::{PhysicalSize, WindowEvent};
use tokio::{
    sync::{oneshot, watch},
    task::LocalSet,
};
use tracing::warn;
use windows::{
    core::Interface,
    Foundation::Numerics::Vector2,
    Win32::{
        Graphics::Dxgi::{
            Common::DXGI_FORMAT_B8G8R8A8_UNORM, IDXGISwapChain1, DXGI_SWAP_CHAIN_FLAG,
        },
        System::{Threading::WaitForSingleObjectEx, WinRT::Composition::ICompositorInterop},
    },
    UI::Composition::{CompositionSurfaceBrush, Compositor, SpriteVisual},
};

use super::{skia_context::SkiaContext, wgpu_context::WGPUContext};

pub struct RenderContext<'a> {
    #[allow(unused)]
    pub frame_buffer_index: usize,
    pub canvas: &'a sk::Canvas,
}

pub type DrawHook = Box<dyn Fn(&mut RenderContext) -> Result<(), anyhow::Error> + Send>;
pub type PreResizeBuffersHook = Box<dyn Fn() -> Result<(), anyhow::Error> + Send>;
pub type PostResizeBuffersHook =
    Box<dyn Fn(&IDXGISwapChain1, PhysicalSize<u32>) -> Result<(), anyhow::Error> + Send>;

pub struct GPUVisual {
    win: Arc<WGuiWindow>,
    _compositor: Compositor,
    visual: SpriteVisual,

    wgpu_context: WGPUContext,
    canvas_context: SkiaContext,

    draw_hook: Vec<DrawHook>,

    pre_resize_buffers_hook: Vec<PreResizeBuffersHook>,
    post_resize_buffers_hook: Vec<PostResizeBuffersHook>,
}

unsafe impl Send for GPUVisual {}

impl GPUVisual {
    pub fn new(
        win: Arc<WGuiWindow>,
        compositor: Compositor,
        size: PhysicalSize<u32>,
    ) -> Result<Self, anyhow::Error> {
        let context = WGPUContext::new(size)?;

        let visual = Self::create_visual(&compositor, &context.swap_chain, size)?;

        let canvas_context = SkiaContext::new(&context)?;

        Ok(Self {
            win,
            _compositor: compositor,
            visual,
            wgpu_context: context,
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

        for i in 0..self.wgpu_context.n_frames {
            unsafe {
                if self.wgpu_context.fench.GetCompletedValue()
                    < self.wgpu_context.fench_values[i as usize]
                {
                    self.wgpu_context.fench.SetEventOnCompletion(
                        self.wgpu_context.fench_values[i as usize],
                        self.wgpu_context.fench_event.clone(),
                    )?;
                    WaitForSingleObjectEx(self.wgpu_context.fench_event, u32::max_value(), false);
                }
            }
        }

        unsafe {
            self.wgpu_context
                .swap_chain
                .ResizeBuffers(
                    self.wgpu_context.n_frames,
                    width,
                    height,
                    DXGI_FORMAT_B8G8R8A8_UNORM,
                    DXGI_SWAP_CHAIN_FLAG(0),
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

        self.wgpu_context.size = size;

        for hook in self.post_resize_buffers_hook.iter() {
            (*hook)(&self.wgpu_context.swap_chain, size)?;
        }

        self.canvas_context.set_surfaces(&self.wgpu_context)?;

        Ok(())
    }

    pub fn run(self) {
        let on_window_resize = {
            let (tx, rx) = watch::channel(self.wgpu_context.size);
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

            let (frame_buffer_index, _frame_buffer) = self.wgpu_context.get_back_frame_buffer()?;

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

            self.wgpu_context.swap_buffer()?;
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
