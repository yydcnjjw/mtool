#![feature(let_chains)]

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

mod event;

#[cfg(feature = "event-loop")]
mod event_loop;

mod keyboard;

pub use crate::{event::*, keyboard::*};

#[cfg(feature = "event-loop")]
pub use event_loop::*;

#[cfg(test)]
mod tests {
    use tracing::debug;
    use tracing_test::traced_test;

    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    #[traced_test]
    async fn test_event_loop() {
        let event_loop = EventLoop::new().unwrap();
        event_loop
            .run(|ev| -> ControlFlow {
                debug!("{:?}", ev);

                ControlFlow::Continue(())
            })
            .await
            .unwrap()
    }
}
