use bevy::{prelude::*, winit::WinitWindows};
use dioxus_desktop::winit::window::Window as WinitWindow;
use mapp::{
    anyhow,
    once_cell::sync::OnceCell,
    tokio::{
        self,
        sync::{mpsc, oneshot},
    },
};
use std::sync::Arc;

static WINDOW_SENDER: OnceCell<mpsc::UnboundedSender<CreateWindowData>> = OnceCell::new();

#[derive(Debug)]
pub struct CreateWindowData {
    pub window_attributes: Window,
    pub window_tx: oneshot::Sender<(Arc<WinitWindow>, Entity)>,
}

pub fn set_window_sender(tx: mpsc::UnboundedSender<CreateWindowData>) {
    _ = WINDOW_SENDER.set(tx);
}

pub async fn create_window(window: Window) -> Result<(Arc<WinitWindow>, Entity), anyhow::Error> {
    let (tx, rx) = oneshot::channel();

    let sender = match WINDOW_SENDER.get() {
        Some(tx) => tx,
        None => &tokio::task::spawn_blocking(move || WINDOW_SENDER.wait().clone()).await?,
    };

    sender.send(CreateWindowData {
        window_attributes: window,
        window_tx: tx,
    })?;

    Ok(rx.await?)
}

#[derive(Resource)]
pub(super) struct WindowFactory {
    rx: mpsc::UnboundedReceiver<CreateWindowData>,
}

impl WindowFactory {
    pub fn new(rx: mpsc::UnboundedReceiver<CreateWindowData>) -> Self {
        Self { rx }
    }
}

pub(super) fn receive_create_window_event(
    mut commands: Commands,
    mut window_factory: ResMut<WindowFactory>,
) {
    match window_factory.rx.try_recv() {
        Ok(CreateWindowData {
            window_attributes,
            window_tx,
        }) => {
            let id = commands.spawn(window_attributes.clone()).id();
            commands.spawn(WindowSender {
                window_id: id,
                tx: Some(window_tx),
            });
        }
        Err(_) => {}
    }
}

#[derive(Component)]
pub struct WindowSender {
    window_id: Entity,
    tx: Option<oneshot::Sender<(Arc<WinitWindow>, Entity)>>,
}

pub(super) fn try_send_window(
    mut commands: Commands,
    mut q: Query<(Entity, &mut WindowSender)>,
    winit_windows: NonSendMut<WinitWindows>,
) {
    for (id, mut sender) in q.iter_mut() {
        if let Some(window) = winit_windows.get_window(sender.window_id) {
            if let Some(tx) = sender.tx.take() {
                if let Err(e) = tx.send((window.clone_window(), sender.window_id)) {
                    warn!("{:?}", e);
                }
            }

            commands.entity(id).remove::<WindowSender>();
        }
    }
}
