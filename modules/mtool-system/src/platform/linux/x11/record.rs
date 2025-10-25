use mapp::{
    anyhow::{self, anyhow, bail},
    futures::{
        future,
        stream::{self, BoxStream},
        StreamExt, TryStreamExt,
    },
    keyboard_types::KeyState,
    tokio,
    tracing::{debug, info},
};
use std::{
    ops::Deref,
    sync::atomic::{AtomicI64, Ordering},
    u8,
};
use x11rb_async::{
    blocking::BlockingConnection,
    connection::{Connection, RequestConnection},
    protocol::{
        record::{self, ConnectionExt as _},
        xkb::{self, ConnectionExt as _},
        xproto,
    },
    rust_connection::RustConnection,
    x11_utils::TryParse,
    XCBConnection,
};

use crate::{
    keyboard::update_modifier_state, platform::linux::x11::keyboard::scancode_to_physicalkey,
    Keyboard, SystemEvent,
};

pub type EventStream<'a> = BoxStream<'a, Result<SystemEvent, anyhow::Error>>;

pub struct RecordSession {
    ctrl_conn: RustConnection,
    data_conn: RustConnection,

    id: AtomicI64,
}

impl RecordSession {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let (ctrl_conn, _, ctrl_drive) = RustConnection::connect(None).await?;
        let (data_conn, _, data_drive) = RustConnection::connect(None).await?;
        tokio::spawn(ctrl_drive);
        tokio::spawn(data_drive);

        // Check if the record extension is supported.
        if ctrl_conn
            .extension_information(record::X11_EXTENSION_NAME)
            .await?
            .is_none()
        {
            bail!("The X11 server does not support the RECORD extension");
        }

        let ver = ctrl_conn
            .record_query_version(
                record::X11_XML_VERSION.0 as _,
                record::X11_XML_VERSION.1 as _,
            )
            .await?
            .reply()
            .await?;

        debug!(
            "requested RECORD extension version {:?}, server supports {:?}",
            record::X11_XML_VERSION,
            (ver.major_version, ver.minor_version)
        );

        Ok(Self {
            ctrl_conn,
            data_conn,
            id: AtomicI64::new(-1),
        })
    }
}

impl RecordSession {
    pub async fn close(&self) -> Result<(), anyhow::Error> {
        let id = self.id.load(Ordering::Relaxed);
        if id != -1 {
            self.ctrl_conn.record_disable_context(id as u32).await?;
            self.ctrl_conn.flush().await?;
        }
        Ok(())
    }

    pub async fn event_stream(&self) -> Result<EventStream<'_>, anyhow::Error> {
        // Set up a recording context
        let rc = self.ctrl_conn.generate_id().await?;
        self.id.store(rc as i64, Ordering::Relaxed);

        let empty = record::Range8 { first: 0, last: 0 };
        let empty_ext = record::ExtRange {
            major: empty,
            minor: record::Range16 { first: 0, last: 0 },
        };

        let range = record::Range {
            core_requests: empty,
            core_replies: empty,
            ext_requests: empty_ext,
            ext_replies: empty_ext,
            delivered_events: empty,
            device_events: record::Range8 {
                // We want notification of core X11 events between key press and motion notify
                first: xproto::KEY_PRESS_EVENT,
                last: xproto::MOTION_NOTIFY_EVENT,
            },
            errors: empty,
            client_started: false,
            client_died: false,
        };
        self.ctrl_conn
            .record_create_context(rc, 0, &[record::CS::ALL_CLIENTS.into()], &[range])
            .await?
            .check()
            .await?;

        Ok(self
            .data_conn
            .record_enable_context(rc)
            .await?
            .map_err(|e| anyhow!("{e}"))
            .try_filter_map(move |reply| {
                if reply.client_swapped {
                    return future::err(anyhow!(
                        "client_swapped={} is unsupported",
                        reply.client_swapped
                    ));
                }

                if reply.category == 0 {
                    // From Server
                    let mut events = Vec::new();
                    let mut remaining = &reply.data[..];
                    while !remaining.is_empty() {
                        let (event, r) = try_parse(remaining).unwrap();
                        remaining = r;
                        if let Some(event) = event {
                            events.push(Ok(event));
                        }
                    }
                    return future::ok(Some(stream::iter(events)));
                } else if reply.category == 5 {
                    // End Of Data
                    return future::err(anyhow!("end of data"));
                } else {
                    return future::ok(None);
                }
            })
            .try_flatten()
            .boxed())
    }
}

fn try_parse<'a, 'b>(data: &'b [u8]) -> Result<(Option<SystemEvent>, &'b [u8]), anyhow::Error> {
    let ev = data[0];
    match ev {
        xproto::KEY_PRESS_EVENT | xproto::KEY_RELEASE_EVENT => {
            let ((event, remaining), state) = if ev == xproto::KEY_PRESS_EVENT {
                (xproto::KeyPressEvent::try_parse(data)?, KeyState::Down)
            } else {
                (xproto::KeyReleaseEvent::try_parse(data)?, KeyState::Up)
            };

            let event =
                scancode_to_physicalkey(event.detail.saturating_sub(8).into()).map(|code| {
                    SystemEvent::Keyboard(Keyboard {
                        state,
                        code,
                        modifiers: update_modifier_state(&code, &state),
                    })
                });

            Ok((event, remaining))
        }
        xproto::BUTTON_PRESS_EVENT => {
            let (_event, remaining) = xproto::ButtonPressEvent::try_parse(data)?;
            Ok((None, remaining))
        }
        xproto::BUTTON_RELEASE_EVENT => {
            let (_event, remaining) = xproto::ButtonReleaseEvent::try_parse(data)?;
            Ok((None, remaining))
        }
        xproto::MOTION_NOTIFY_EVENT => {
            let (_event, remaining) = xproto::MotionNotifyEvent::try_parse(data)?;
            Ok((None, remaining))
        }
        0 => {
            // This is a reply, we compute its length as follows
            let (length, _) = u32::try_parse(&data[4..])?;
            let length = usize::try_from(length).unwrap() * 4 + 32;
            Ok((None, &data[length..]))
        }
        _ => Ok((None, &data[32..])),
    }
}
