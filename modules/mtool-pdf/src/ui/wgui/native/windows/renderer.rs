use mtool_wgui::WGuiWindow;
use std::{mem::size_of, sync::Arc};
use tauri::PhysicalSize;
use tokio::sync::oneshot;
use windows::{
    core::ComInterface, Foundation::Numerics::{Vector2, Vector3}, System::DispatcherQueueController, Win32::{
        Foundation::BOOL,
        Graphics::Dxgi::IDXGISwapChain1,
        System::WinRT::{
            Composition::ICompositorDesktopInterop, CreateDispatcherQueueController,
            DispatcherQueueOptions, DQTAT_COM_ASTA, DQTYPE_THREAD_CURRENT,
        },
    }, UI::Composition::{Compositor, ContainerVisual, Desktop::DesktopWindowTarget}
};

use super::{
    d3d12_visual::{
        D3d12Visual, DrawHook, PostResizeBuffersHook, PreResizeBuffersHook, RenderContext,
    },
    webview_visual::WebviewVisual,
};

pub struct RendererBuilder {
    win: Arc<WGuiWindow>,

    draw_hook: Vec<DrawHook>,
    pre_resize_buffers_hook: Vec<PreResizeBuffersHook>,
    post_resize_buffers_hook: Vec<PostResizeBuffersHook>,
}

impl RendererBuilder {
    pub fn new(win: Arc<WGuiWindow>) -> Self {
        Self {
            win,
            draw_hook: Vec::new(),
            pre_resize_buffers_hook: Vec::new(),
            post_resize_buffers_hook: Vec::new(),
        }
    }

    pub fn add_draw_hook<Hook>(mut self, hook: Hook) -> Self
    where
        Hook: Fn(&mut RenderContext) -> Result<(), anyhow::Error> + Send + 'static,
    {
        self.draw_hook.push(Box::new(hook));
        self
    }

    #[allow(unused)]
    pub fn add_resize_buffers_hook<PreHook, PostHook>(
        mut self,
        pre_hook: PreHook,
        post_hook: PostHook,
    ) -> Self
    where
        PreHook: Fn() -> Result<(), anyhow::Error> + Send + 'static,
        PostHook:
            Fn(&IDXGISwapChain1, PhysicalSize<u32>) -> Result<(), anyhow::Error> + Send + 'static,
    {
        self.pre_resize_buffers_hook.push(Box::new(pre_hook));
        self.post_resize_buffers_hook.push(Box::new(post_hook));
        self
    }

    pub async fn build(self) -> Result<Renderer, anyhow::Error> {
        Renderer::new(self).await
    }
}

#[allow(unused)]
pub struct Renderer {
    win: Arc<WGuiWindow>,

    compositor: Compositor,
    target: DesktopWindowTarget,
    dispatcher_queue_controller: DispatcherQueueController,
    root_visual: ContainerVisual,

    webview_visual: WebviewVisual,
}

impl Renderer {
    pub async fn new(builder: RendererBuilder) -> Result<Self, anyhow::Error> {
        let RendererBuilder {
            win,
            draw_hook,
            pre_resize_buffers_hook,
            post_resize_buffers_hook,
        } = builder;

        let hwnd = win.hwnd()?;
        let (compositor, target, dispatcher_queue_controller, root_visual) = {
            let (tx, rx) = oneshot::channel();
            let hwnd = hwnd.clone();
            win.run_on_main_thread(move || {
                let _ = tx.send(move || -> Result<_, anyhow::Error> {
                    unsafe {
                        let dispatcher_queue_controller =
                            CreateDispatcherQueueController(DispatcherQueueOptions {
                                dwSize: size_of::<DispatcherQueueOptions>() as u32,
                                threadType: DQTYPE_THREAD_CURRENT,
                                apartmentType: DQTAT_COM_ASTA,
                            })?;

                        let compositor = Compositor::new()?;

                        let target = compositor
                            .cast::<ICompositorDesktopInterop>()?
                            .CreateDesktopWindowTarget(hwnd, BOOL::from(false))?;

                        let root_visual = {
                            let root = compositor.CreateContainerVisual()?;
                            root.SetRelativeSizeAdjustment(Vector2 { X: 1., Y: 1. })?;
                            root.SetOffset(Vector3 {
                                X: 0.,
                                Y: 0.,
                                Z: 0.,
                            })?;
                            target.SetRoot(&root)?;
                            root
                        };
                        Ok((compositor, target, dispatcher_queue_controller, root_visual))
                    }
                }());
            })?;
            rx.await??
        };

        let size = win.inner_size()?;
        let webview_visual = WebviewVisual::new(win.clone(), &compositor).await?;
        webview_visual.set_size(size)?;

        let mut d3d12_visual = D3d12Visual::new(win.clone(), compositor.clone(), size)?;
        d3d12_visual.set_draw_hooks(draw_hook);
        d3d12_visual.set_resize_buffers_hooks(pre_resize_buffers_hook, post_resize_buffers_hook);

        root_visual
            .Children()?
            .InsertAtTop(webview_visual.handle())?;
        root_visual
            .Children()?
            .InsertAtBottom(d3d12_visual.handle())?;

        d3d12_visual.run();

        Ok(Self {
            win,
            compositor,
            target,
            dispatcher_queue_controller,
            root_visual,
            webview_visual,
        })
    }
}
