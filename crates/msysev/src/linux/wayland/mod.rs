use std::future::poll_fn;
use tokio::{select, sync::mpsc};
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

struct Context {
    seat: Option<WlSeat>,
    data_control_manager: Option<ZwlrDataControlManagerV1>,
    data_control_device: Option<ZwlrDataControlDeviceV1>,
}

impl Context {
    fn handle(&mut self, _: AppEvent) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

impl Dispatch<WlSeat, ()> for Context {
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

impl Dispatch<ZwlrDataControlDeviceV1, ()> for Context {
    fn event(
        _state: &mut Self,
        _proxy: &ZwlrDataControlDeviceV1,
        event: <ZwlrDataControlDeviceV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_data_control_device_v1::Event::DataOffer { id } => {
                println!("{:?}", id);
            }
            zwlr_data_control_device_v1::Event::Selection { id: Some(id) } => {
                println!("selection: {:?}", id);
            }
            zwlr_data_control_device_v1::Event::Finished => {
                println!("finished")
            }
            zwlr_data_control_device_v1::Event::PrimarySelection { id: Some(id) } => {
                println!("primary selection: {:?}", id);
            }
            _ => {}
        }
    }

    event_created_child!(Context, ZwlrDataControlDeviceV1, [
        zwlr_data_control_device_v1::EVT_DATA_OFFER_OPCODE => (ZwlrDataControlOfferV1, ())
    ]);
}

impl Dispatch<ZwlrDataControlOfferV1, ()> for Context {
    fn event(
        ctx: &mut Self,
        _proxy: &ZwlrDataControlOfferV1,
        event: <ZwlrDataControlOfferV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        if let zwlr_data_control_offer_v1::Event::Offer { mime_type } = event {
            println!("{}", mime_type);
        }
    }
}

impl Dispatch<ZwlrDataControlManagerV1, ()> for Context {
    fn event(
        _state: &mut Self,
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

impl Dispatch<wl_registry::WlRegistry, ()> for Context {
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

            println!("[{}] {} (v{})", name, interface, version);
        }
    }
}

enum AppEvent {}

fn run(ctx: &mut Context) -> Result<(), anyhow::Error> {
    let conn = Connection::connect_to_env()?;

    let display = conn.display();

    let mut queue = conn.new_event_queue();

    let qh = queue.handle();
    let _registry = display.get_registry(&qh, ());

    loop {
        queue.blocking_dispatch(ctx)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event() {
        let mut ctx = Context {
            seat: None,
            data_control_manager: None,
            data_control_device: None,
        };

        run(&mut ctx).unwrap()
    }
}
