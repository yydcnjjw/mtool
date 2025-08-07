use mapp::{
    android::{
        android_log_sys::{self, LogPriority},
        LogcatWriter,
    },
    android_activity::AndroidApp,
    anyhow::{self, Context},
    once_cell::sync::OnceCell,
    prelude::*,
    tokio::sync::broadcast,
    tracing::{debug, info, warn},
};
use mtool_dioxus::{
    desktop::wry::prelude::{jni::objects::JValue, *},
    prelude::*,
};

use crate::{Notification, SystenEvent};

pub struct SystemEventSource {
    source: broadcast::Sender<SystenEvent>,
}

static SOURCE: OnceCell<broadcast::Sender<SystenEvent>> = OnceCell::new();

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

    pub fn subscribe(&self) -> broadcast::Receiver<SystenEvent> {
        self.source.subscribe()
    }

    pub fn sender(&self) -> broadcast::Sender<SystenEvent> {
        self.source.clone()
    }    
}

android_fn![
    org_yydcnjjw_mtool,
    system,
    NotificationListener,
    onNotificationPostedNative,
    [JString, JString]
];

#[allow(non_snake_case)]
pub unsafe fn onNotificationPostedNative(
    mut jenv: JNIEnv,
    _: JClass,
    pkg_name: JString,
    _channel_id: JString,
) {
    match jenv.get_string(&pkg_name) {
        Ok(pkg_name) => {
            if let Some(source) = SOURCE.get() {
                if let Err(e) = source.send(SystenEvent::NotificationPosted(Notification {
                    package_name: pkg_name.to_string_lossy().to_string(),
                })) {
                    warn!("Failed to send {}", e);
                }
            } else {
                warn!("Failed to get source");
            }
        }
        Err(e) => {
            warn!("Failed to parse JString: {}", e);
        }
    }
}
