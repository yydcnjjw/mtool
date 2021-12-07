use crate::app::App;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg(target_os = "windows")]
use winit::platform::windows::EventLoopExtWindows;

#[cfg(target_os = "linux")]
use winit::platform::unix::EventLoopExtUnix;

pub async fn module_load(app: &App) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || {
        let event_loop: EventLoop<()> = EventLoop::new_any_thread();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window.id() => *control_flow = ControlFlow::Exit,
                _ => (),
            }
        });
    });

    Ok(())
}
