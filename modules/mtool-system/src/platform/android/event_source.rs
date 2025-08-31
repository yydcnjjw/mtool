use mapp::{
    android::{
        android_log_sys::{self, LogPriority},
        LogcatWriter,
    },
    android_activity::AndroidApp,
    anyhow::{self, Context},
    once_cell::sync::OnceCell,
    prelude::*,
    serde_json,
    tokio::sync::broadcast,
    tracing::{debug, info, warn},
};
use mtool_dioxus::{
    desktop::wry::prelude::{jni::objects::JValue, *},
    prelude::*,
};

use crate::{Notification, SystemEvent};

pub struct SystemEventSource {
    source: broadcast::Sender<SystemEvent>,
}

static SOURCE: OnceCell<broadcast::Sender<SystemEvent>> = OnceCell::new();

impl SystemEventSource {
    pub async fn new(context: Res<DioxusContext>) -> Result<SystemEventSource, anyhow::Error> {
        let (source, _) = broadcast::channel(64);

        if let Err(e) = SOURCE.set(source.clone()) {
            warn!("Failed to set {:?}", e);
        }

        context
            .run_on_main_thread(move |ctx| {
                let vm = ctx.jvm().context("get java vm")?;
                let mut env = vm.get_env().context("get java env")?;
                let activity =
                    unsafe { JObject::from_raw(ctx.android_app().activity_as_ptr() as _) };

                let notification_listener_class = find_class(
                    &mut env,
                    &activity,
                    "org/yydcnjjw/mtool/system/NotificationListener".into(),
                )
                .context("find_class org/yydcnjjw/mtool/system/NotificationListener failed")?;

                env.call_static_method(
                    notification_listener_class,
                    "start",
                    "(Lorg/yydcnjjw/mtool/dioxus/MainActivity;)V",
                    &[JValue::Object(&activity)],
                )
                .context("cal static method start (Lorg/yydcnjjw/mtool/dioxus/MainActivity;)V")?;

                Ok(())
            })
            .await?;

        Ok(SystemEventSource { source })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.source.subscribe()
    }

    pub fn sender(&self) -> broadcast::Sender<SystemEvent> {
        self.source.clone()
    }
}

android_fn![
    org_yydcnjjw_mtool,
    system,
    NotificationListener,
    onNotificationPostedNative,
    [JString]
];

#[allow(non_snake_case)]
pub unsafe fn onNotificationPostedNative(mut jenv: JNIEnv, _: JClass, data: JString) {
    match jenv.get_string(&data) {
        Ok(data) => match serde_json::from_slice(data.to_bytes()) {
            Ok(notification) => {
                if let Some(source) = SOURCE.get() {
                    if let Err(e) = source.send(SystemEvent::NotificationPosted(notification)) {
                        warn!("Failed to send {}", e);
                    }
                } else {
                    warn!("Failed to get source");
                }
            }
            Err(e) => {
                warn!("{:?}", e);
            }
        },
        Err(e) => {
            warn!("Failed to parse JString: {}", e);
        }
    }
}
