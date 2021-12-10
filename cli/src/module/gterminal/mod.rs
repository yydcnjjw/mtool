mod controls;
mod scene;
mod terminal;

use std::ops::Deref;

use crate::{app::App, core::evbus::Sender};

use terminal::Terminal;

use iced_wgpu::{wgpu, Backend, Renderer, Settings, Viewport};
use iced_winit::{
    clipboard, command, conversion, futures, program,
    winit::{
        self,
        dpi::{PhysicalSize, Position},
        monitor::MonitorHandle,
    },
    Clipboard, Color, Debug, Error, Executor, Proxy, Runtime, Size,
};

use futures::{task::SpawnExt, SinkExt};
use winit::{
    dpi::PhysicalPosition,
    event::{Event, ModifiersState, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

#[cfg(target_os = "windows")]
use iced_winit::winit::platform::windows::EventLoopExtWindows;

#[cfg(target_os = "linux")]
use iced_winit::winit::platform::unix::{EventLoopExtUnix, WindowBuilderExtUnix, XWindowType};

use self::{controls::Controls, scene::Scene};

type TokioHandle = tokio::runtime::Handle;

struct CurrentTokio {
    handle: tokio::runtime::Handle,
}

impl Executor for CurrentTokio {
    fn new() -> Result<Self, futures::io::Error>
    where
        Self: Sized,
    {
        Ok(Self {
            handle: tokio::runtime::Handle::current(),
        })
    }

    fn spawn(&self, future: impl futures::Future<Output = ()> + Send + 'static) {
        self.handle.spawn(future);
    }

    fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        self.handle.enter();
        f()
    }
}

pub async fn run(tx: Sender) -> anyhow::Result<()> {
    // Initialize winit
    let event_loop = EventLoop::<terminal::Message>::new_any_thread();

    let proxy = event_loop.create_proxy();
    let mut runtime = {
        let proxy = Proxy::new(event_loop.create_proxy());

        let executor = CurrentTokio::new().map_err(Error::ExecutorCreationFailed)?;

        Runtime::new(executor, proxy)
    };

    let mut window_builder = winit::window::WindowBuilder::new();

    #[cfg(target_os = "linux")]
    {
        window_builder = window_builder
            .with_x11_window_type(vec![XWindowType::Toolbar])
            .with_transparent(true)
    }

    let primary = event_loop
        .primary_monitor()
        .ok_or(anyhow::anyhow!("primary monitor is not found"))?;

    let primary_size = primary.size();

    let window_size = PhysicalSize::<u32>::new(1024, 128);

    let window_pos = PhysicalPosition::<i32>::new(
        ((primary_size.width - window_size.width) / 2).try_into()?,
        8,
    );

    let window = window_builder
        .with_inner_size(window_size)
        .with_position(window_pos)
        .with_decorations(false)
        .build(&event_loop)?;

    let physical_size = window.inner_size();
    let mut viewport = Viewport::with_physical_size(
        Size::new(physical_size.width, physical_size.height),
        window.scale_factor(),
    );
    let mut cursor_position = PhysicalPosition::new(-1.0, -1.0);
    let mut modifiers = ModifiersState::default();
    let mut clipboard = Clipboard::connect(&window);

    // Initialize wgpu
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };

    let (format, (mut device, queue)) = futures::executor::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Request adapter");

        (
            surface
                .get_preferred_format(&adapter)
                .expect("Get preferred format"),
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        features: wgpu::Features::empty(),
                        limits: wgpu::Limits::default(),
                    },
                    None,
                )
                .await
                .expect("Request device"),
        )
    });

    {
        let size = window.inner_size();

        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Mailbox,
            },
        )
    };
    let mut resized = false;

    // Initialize staging belt and local pool
    let mut staging_belt = wgpu::util::StagingBelt::new(5 * 1024);
    let mut local_pool = futures::executor::LocalPool::new();

    // Initialize scene and GUI controls
    let scene = Scene::new(&mut device);
    let controls = Terminal::new(tx);

    // Initialize iced
    let mut debug = Debug::new();

    let mut renderer = Renderer::new(Backend::new(&mut device, Settings::default(), format));

    let mut state =
        program::State::new(controls, viewport.logical_size(), &mut renderer, &mut debug);

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        // You should change this if you want to render continuosly
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(message) => {
                state.queue_message(message);
            }
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        cursor_position = position;
                    }
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = new_modifiers;
                    }
                    WindowEvent::Resized(new_size) => {
                        viewport = Viewport::with_physical_size(
                            Size::new(new_size.width, new_size.height),
                            window.scale_factor(),
                        );

                        resized = true;
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }

                // Map window event to iced event
                if let Some(event) =
                    iced_winit::conversion::window_event(&event, window.scale_factor(), modifiers)
                {
                    state.queue_event(event);
                }
            }
            Event::MainEventsCleared => {
                // If there are events pending
                if !state.is_queue_empty() {
                    // We update iced
                    if let Some(cmd) = state.update(
                        viewport.logical_size(),
                        conversion::cursor_position(cursor_position, viewport.scale_factor()),
                        &mut renderer,
                        &mut clipboard,
                        &mut debug,
                    ) {
                        for action in cmd.actions() {
                            match action {
                                command::Action::Future(future) => {
                                    runtime.spawn(future);
                                }
                                command::Action::Clipboard(action) => match action {
                                    clipboard::Action::Read(tag) => {
                                        let message = tag(clipboard.read());

                                        proxy
                                            .send_event(message)
                                            .expect("Send message to event loop");
                                    }
                                    clipboard::Action::Write(contents) => {
                                        clipboard.write(contents);
                                    }
                                },
                                command::Action::Window(_) => todo!(),
                                // command::Action::Window(action) => match action {
                                //     window::Action::Resize { width, height } => {
                                //         window.set_inner_size(winit::dpi::LogicalSize {
                                //             width,
                                //             height,
                                //         });
                                //     }
                                //     window::Action::Move { x, y } => {
                                //         window.set_outer_position(winit::dpi::LogicalPosition {
                                //             x,
                                //             y,
                                //         });
                                //     }
                                // },
                            }
                        }
                    }
                    // and request a redraw
                    window.request_redraw();
                }
            }
            Event::RedrawRequested(_) => {
                if resized {
                    let size = window.inner_size();

                    surface.configure(
                        &device,
                        &wgpu::SurfaceConfiguration {
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            format,
                            width: size.width,
                            height: size.height,
                            present_mode: wgpu::PresentMode::Mailbox,
                        },
                    );

                    resized = false;
                }

                match surface.get_current_texture() {
                    Ok(frame) => {
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });

                        let program = state.program();

                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        {
                            // We clear the frame
                            let mut render_pass =
                                scene.clear(&view, &mut encoder, Color::TRANSPARENT);

                            // Draw the scene
                            // scene.draw(&mut render_pass);
                        }

                        // And then iced on top
                        renderer.with_primitives(|backend, primitive| {
                            backend.present(
                                &mut device,
                                &mut staging_belt,
                                &mut encoder,
                                &view,
                                primitive,
                                &viewport,
                                &debug.overlay(),
                            );
                        });

                        // Then we submit the work
                        staging_belt.finish();
                        queue.submit(Some(encoder.finish()));
                        frame.present();

                        // Update the mouse cursor
                        window.set_cursor_icon(iced_winit::conversion::mouse_interaction(
                            state.mouse_interaction(),
                        ));

                        // And recall staging buffers
                        local_pool
                            .spawner()
                            .spawn(staging_belt.recall())
                            .expect("Recall staging buffers");

                        local_pool.run_until_stalled();
                    }
                    Err(error) => match error {
                        wgpu::SurfaceError::OutOfMemory => {
                            panic!("Swapchain error: {}. Rendering cannot continue.", error)
                        }
                        _ => {
                            // Try rendering again next frame.
                            window.request_redraw();
                        }
                    },
                }
            }
            _ => {}
        }
    })
}

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    tokio::spawn(run(app.evbus.sender()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run() {}
}
