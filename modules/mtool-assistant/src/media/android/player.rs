use mapp::{
    anyhow::{self, Context},
    dashmap::DashMap,
    futures::{StreamExt, TryStreamExt},
    once_cell::sync::Lazy,
    prelude::*,
    serde_json,
    tokio::{
        self,
        sync::{broadcast, oneshot},
    },
    tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream},
    tracing::{debug, info, warn},
    url::Url,
};
use mtool_dioxus::{
    desktop::wry::prelude::{android_fn, jni::objects::JValue, *},
    prelude::DioxusContext,
};

use crate::media::{
    android::once_callback::new_once_callback, MediaItem, Player, PlayerEvent, PlayerEventStream,
};

use super::callback::new_callback;

static MEDIA_ITEMS: Lazy<DashMap<String, MediaItem>> = Lazy::new(|| DashMap::new());

#[derive(Clone)]
pub struct MediaPlayer {
    ctx: DioxusContext,
    controller: GlobalRef,

    sender: broadcast::Sender<PlayerEvent>,
}

#[async_trait]
impl Player for MediaPlayer {
    async fn play(&self) -> Result<(), anyhow::Error> {
        self.with_env(|mut env, _| {
            _ = env.call_method(self.controller.as_obj(), "play", "()V", &[])?;
            Ok(())
        })
    }

    async fn pause(&self) -> Result<(), anyhow::Error> {
        self.with_env(|mut env, _| {
            _ = env.call_method(self.controller.as_obj(), "pause", "()V", &[])?;
            Ok(())
        })
    }

    async fn volume(&self) -> Result<f64, anyhow::Error> {
        self.with_env(|mut env, _| {
            Ok(env
                .call_method(self.controller.as_obj(), "getVolume", "()F", &[])?
                .f()? as f64)
        })
    }

    async fn set_volume(&self, value: f64) -> Result<(), anyhow::Error> {
        self.with_env(|mut env, _| {
            env.call_method(
                self.controller.as_obj(),
                "setVolume",
                "(F)V",
                &[(value as f32).into()],
            )?;
            Ok(())
        })
    }

    async fn set_media_items(&self, items: Vec<MediaItem>) -> Result<(), anyhow::Error> {
        MEDIA_ITEMS.clear();

        self.with_env(|mut env, _| {
            let array =
                env.new_object_array(items.len() as i32, "java/lang/String", JObject::null())?;

            for (i, item) in items.iter().enumerate() {
                env.set_object_array_element(
                    &array,
                    i as i32,
                    env.new_string(serde_json::to_string(item)?)?,
                )?;
            }

            env.call_method(
                self.controller.as_obj(),
                "setMediaItems",
                "([Ljava/lang/String;)V",
                &[JValue::Object(&array)],
            )?;

            for item in items {
                MEDIA_ITEMS.insert(item.id.clone(), item);
            }
            Ok(())
        })
    }

    async fn current_media_item(&self) -> Result<Option<MediaItem>, anyhow::Error> {
        let (tx, rx) = oneshot::channel();
        self.with_env(|mut env, activity| {
            let handler = new_once_callback(&mut env, &activity, |id: String| {
                _ = tx.send(id);
            })?;

            env.call_method(
                self.controller.as_obj(),
                "currentMediaItem",
                "(Lorg/yydcnjjw/mtool/assistant/OnceCallback;)V",
                &[JValue::Object(&handler)],
            )?;
            Ok(())
        })?;
        let uri = rx.await?;

        Ok(MEDIA_ITEMS.get(&uri).map(|item| item.to_owned()))
    }

    async fn listen(&self) -> Result<PlayerEventStream, anyhow::Error> {
        let tx = self.sender.clone();
        self.with_env(|mut env, activity| {
            let callback =
                new_callback(
                    &mut env,
                    &activity,
                    move |data: String| match serde_json::from_str(&data) {
                        Ok(ev) => _ = tx.send(ev),
                        Err(e) => warn!("{e:?}"),
                    },
                )?;

            env.call_method(
                self.controller.as_obj(),
                "listen",
                "(Lorg/yydcnjjw/mtool/assistant/Callback;)V",
                &[JValue::Object(&callback)],
            )?;
            Ok(())
        })?;

        Ok(BroadcastStream::new(self.sender.subscribe())
            .map_err(|e| match e {
                BroadcastStreamRecvError::Lagged(n) => broadcast::error::RecvError::Lagged(n),
            })
            .boxed())
    }
}

impl MediaPlayer {
    pub async fn new(ctx: DioxusContext) -> Result<Self, anyhow::Error> {
        let rx = {
            let vm = ctx.jvm().context("get java vm")?;
            let mut env = vm.get_env()?;
            let activity = unsafe { JObject::from_raw(ctx.android_app().activity_as_ptr() as _) };

            let controller_class = find_class(
                &mut env,
                &activity,
                "org/yydcnjjw/mtool/assistant/PlaybackController".into(),
            )?;

            let (tx, rx) = oneshot::channel();
            let handler = new_once_callback(&mut env, &activity, |controller: GlobalRef| {
                _ = tx.send(controller);
            })?;

            env.call_static_method(
                controller_class,
                "connect",
                "(Landroid/content/Context;Lorg/yydcnjjw/mtool/assistant/OnceCallback;)V",
                &[JValue::Object(&activity), JValue::Object(&handler)],
            )?;

            rx
        };

        let controller = rx.await?;

        info!("MediaPlayer is ready");

        let (sender, _) = broadcast::channel(16);

        Ok(Self {
            ctx,
            controller,
            sender,
        })
    }

    fn with_env<F, O>(&self, f: F) -> Result<O, anyhow::Error>
    where
        F: for<'local> FnOnce(JNIEnv<'local>, JObject<'local>) -> Result<O, anyhow::Error>,
    {
        let vm = self.ctx.jvm().context("get java vm")?;
        let env = vm.get_env()?;
        let activity = unsafe { JObject::from_raw(self.ctx.android_app().activity_as_ptr() as _) };
        f(env, activity)
    }
}

android_fn![
    org_yydcnjjw_mtool,
    assistant,
    PlaybackService,
    resolveUri,
    [JString<'local>],
    JString<'local>
];

#[allow(non_snake_case)]
unsafe fn resolveUri<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    uri: JString<'local>,
) -> JString<'local> {
    env.get_string(&uri)
        .context("parse uri")
        .and_then(|uri| {
            let uri = uri.to_string_lossy().to_string();
            tokio::runtime::Builder::new_current_thread()
                .build()?
                .block_on(resolve_uri(uri))
        })
        .and_then(|uri| env.new_string(&uri).context(format!("new_string {uri}")))
        .unwrap_or_else(|e| {
            warn!("{e:?}");
            JString::default()
        })
}

async fn resolve_uri(uri_raw: String) -> Result<String, anyhow::Error> {
    debug!("resolve {uri_raw}");

    let uri = Url::parse(&uri_raw)?;

    if uri.scheme() == "mtool" {
        MEDIA_ITEMS
            .get(uri.authority())
            .context(format!("{uri} not found"))?
            .resolve_uri(uri)
            .await
    } else {
        Ok(uri_raw)
    }
}
