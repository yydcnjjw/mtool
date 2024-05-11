use std::os::fd::AsFd;

use crate::{Event, SelectionEvent};
use mime::Mime;
use tokio::{
    io::AsyncReadExt,
    net::unix::pipe::pipe,
    sync::mpsc,
};
use tracing::{debug, warn};
use wayland_client::{
    event_created_child,
    protocol::{
        wl_registry,
        wl_seat::{self, WlSeat},
    },
    Connection, Dispatch, Proxy, QueueHandle,
};
use wayland_protocols_wlr::data_control::v1::client::{
    zwlr_data_control_device_v1::{self, ZwlrDataControlDeviceV1},
    zwlr_data_control_manager_v1::ZwlrDataControlManagerV1,
    zwlr_data_control_offer_v1::{self, ZwlrDataControlOfferV1},
};

pub struct State {
    seat: Option<WlSeat>,
    data_control_manager: Option<ZwlrDataControlManagerV1>,
    data_control_device: Option<ZwlrDataControlDeviceV1>,

    sender: mpsc::UnboundedSender<Event>,

    mime_types: Vec<Mime>,
}

impl State {
    pub fn new(sender: mpsc::UnboundedSender<Event>) -> Self {
        Self {
            seat: None,
            data_control_manager: None,
            data_control_device: None,
            sender,
            mime_types: Vec::new(),
        }
    }

    fn handle_selection(
        &mut self,
        _primary: bool,
        id: ZwlrDataControlOfferV1,
    ) -> Result<(), anyhow::Error> {
        let (tx, mut rx) = pipe()?;

        let mime_type = self.mime_types.pop().unwrap_or(mime::TEXT_PLAIN_UTF_8);
        self.mime_types.clear();

        id.receive(
            mime_type.to_string(),
            tx.into_blocking_fd().unwrap().as_fd(),
        );

        let sender = self.sender.clone();
        tokio::spawn(async move {
            let mut buf = Vec::new();
            match rx.read_to_end(&mut buf).await {
                Ok(_) => {
                    let _ = sender.send(Event::Selection(SelectionEvent {
                        data: buf,
                        mime_type,
                    }));
                }
                Err(e) => {
                    warn!("{:?}", e);
                }
            }
        });

        id.destroy();
        Ok(())
    }
}

impl Dispatch<WlSeat, ()> for State {
    fn event(
        ctx: &mut Self,
        _proxy: &WlSeat,
        event: <WlSeat as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_seat::Event::Name { name: _ } => {
                if let Some(seat) = &ctx.seat
                    && let Some(data_control_manager) = &ctx.data_control_manager
                {
                    ctx.data_control_device =
                        Some(data_control_manager.get_data_device(seat, qh, ()))
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<ZwlrDataControlDeviceV1, ()> for State {
    fn event(
        ctx: &mut Self,
        _proxy: &ZwlrDataControlDeviceV1,
        event: <ZwlrDataControlDeviceV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        if let Err(e) = match event {
            // zwlr_data_control_device_v1::Event::DataOffer { id: _ } => Ok(()),
            zwlr_data_control_device_v1::Event::Selection { id: Some(id) } => {
                ctx.handle_selection(false, id)
            }
            zwlr_data_control_device_v1::Event::PrimarySelection { id: Some(id) } => {
                ctx.handle_selection(false, id)
            }
            _ => Ok(()),
        } {
            warn!("{:?}", e);
        }
    }

    event_created_child!(State, ZwlrDataControlDeviceV1, [
        zwlr_data_control_device_v1::EVT_DATA_OFFER_OPCODE => (ZwlrDataControlOfferV1, ())
    ]);
}

impl Dispatch<ZwlrDataControlOfferV1, ()> for State {
    fn event(
        ctx: &mut Self,
        _proxy: &ZwlrDataControlOfferV1,
        event: <ZwlrDataControlOfferV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        if let zwlr_data_control_offer_v1::Event::Offer { mime_type } = event {
            if let Ok(mime) = mime_type.parse() {
                ctx.mime_types.push(mime);
            }
        }
    }
}

impl Dispatch<ZwlrDataControlManagerV1, ()> for State {
    fn event(
        _ctx: &mut Self,
        _proxy: &ZwlrDataControlManagerV1,
        event: <ZwlrDataControlManagerV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            _ => {}
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        ctx: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match &interface {
                i if i == WlSeat::interface().name => {
                    ctx.seat = Some(registry.bind::<WlSeat, _, _>(name, version, qh, ()));
                }
                i if i == ZwlrDataControlManagerV1::interface().name => {
                    ctx.data_control_manager = Some(
                        registry.bind::<ZwlrDataControlManagerV1, _, _>(name, version, qh, ()),
                    );
                }
                _ => {}
            }

            debug!("[{}] {} (v{})", name, interface, version);
        }
    }
}
