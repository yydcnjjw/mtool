mod state;

mod event_loop;

pub use event_loop::*;

// use std::{
//     borrow::Borrow, cell::OnceCell, fs::File, future::poll_fn, io::Read, os::fd::{AsFd, FromRawFd, IntoRawFd}, sync::atomic::{AtomicBool, Ordering}
// };
// use tokio::{
//     io::AsyncReadExt,
//     net::unix::pipe::{self, pipe, Receiver},
//     select,
//     sync::mpsc,
// };
// use tracing::{debug, warn};
// use wayland_client::{
//     event_created_child,
//     protocol::{
//         wl_registry,
//         wl_seat::{self, WlSeat},
//     },
//     Connection, Dispatch, Proxy, QueueHandle,
// };
// use wayland_protocols_wlr::data_control::v1::client::{
//     zwlr_data_control_device_v1::{self, ZwlrDataControlDeviceV1},
//     zwlr_data_control_manager_v1::ZwlrDataControlManagerV1,
//     zwlr_data_control_offer_v1::{self, ZwlrDataControlOfferV1},
//     zwlr_data_control_source_v1::{self, ZwlrDataControlSourceV1},
// };

// enum Event {
//     PrimarySelection { data: Vec<u8>, mime_type: String },
//     Selection { data: Vec<u8>, mime_type: String },
// }

// struct Context {
//     seat: Option<WlSeat>,
//     data_control_manager: Option<ZwlrDataControlManagerV1>,
//     data_control_device: Option<ZwlrDataControlDeviceV1>,

//     sender: mpsc::Sender<Event>,

//     mime_types: Vec<String>,

//     need_quit: AtomicBool,
// }

// impl Context {
//     fn new(sender: mpsc::Sender<Event>) -> Self {
//         Self {
//             seat: None,
//             data_control_manager: None,
//             data_control_device: None,
//             sender,
//             mime_types: Vec::new(),
//             need_quit: AtomicBool::new(false),
//         }
//     }

//     fn quit(&self) {
//         self.need_quit.store(true, Ordering::Relaxed);
//     }

//     fn handle_selection(
//         &mut self,
//         primary: bool,
//         id: ZwlrDataControlOfferV1,
//     ) -> Result<(), anyhow::Error> {
//         let (tx, mut rx) = pipe()?;

//         let mime_type = self
//             .mime_types
//             .pop()
//             .unwrap_or("text/plain;charset=utf-8".into());
//         self.mime_types.clear();

//         id.receive(mime_type.clone(), tx.into_blocking_fd().unwrap().as_fd());

//         let sender = self.sender.clone();
//         tokio::spawn(async move {
//             let mut buf = Vec::new();
//             match rx.read_to_end(&mut buf).await {
//                 Ok(_) => {
//                     let _ = sender.send(if primary {
//                         Event::PrimarySelection {
//                             data: buf,
//                             mime_type,
//                         }
//                     } else {
//                         Event::Selection {
//                             data: buf,
//                             mime_type,
//                         }
//                     });
//                 }
//                 Err(e) => {
//                     warn!("{:?}", e);
//                 }
//             }
//         });

//         id.destroy();
//         Ok(())
//     }
// }

// fn run(ctx: &mut Context) -> Result<(), anyhow::Error> {
//     let conn = Connection::connect_to_env()?;

//     let display = conn.display();

//     let mut queue = conn.new_event_queue();

//     let qh = queue.handle();
//     let _registry = display.get_registry(&qh, ());

//     while ctx.need_quit.fetch_not(Ordering::Relaxed) {
//         queue.blocking_dispatch(ctx)?;
//     }

//     Ok(())
// }

// static CONTEXT: OnceCell<Context> = OnceCell::new();

// pub fn quit() -> Result<(), anyhow::Error> {
//     let ctx = CONTEXT.get().unwrap();
//     ctx.quit();
//     Ok(())
// }

// pub fn run_loop(cb: BoxedEventCallback) -> Result<(), anyhow::Error> {
//     if let Err(_) = CONTEXT.set(Context::new(cb)?) {
//         return ;
//     }

//     let mut ctx = CONTEXT.get().unwrap();
//     run(&mut ctx)
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test(flavor = "multi_thread")]
//     async fn test_event() {
//         let (tx, _) = mpsc::channel(1024);
//         let mut ctx = Context::new(tx);
//         run(&mut ctx).unwrap()
//     }
// }
