use std::{os::raw::c_void, ptr::null, sync::Arc};

use mapp::{
    anyhow::{self, Context},
    itertools::Itertools,
    prelude::*,
    tokio::sync::oneshot,
    tracing::{info, warn},
};
use mtool_dioxus::{
    desktop::wry::prelude::{
        android_fn,
        jni::{
            objects::JValue,
            sys::{jfloat, jlong},
        },
        *,
    },
    prelude::DioxusContext,
};

use super::{MediaItem, Netease, Player, PlayerEventStream, Playlist};

#[derive(Clone)]
pub struct MediaPlayer {
    ctx: DioxusContext,
    controller: GlobalRef,
}

#[async_trait]
impl Player for MediaPlayer {
    async fn play(&self) -> Result<(), anyhow::Error> {
        self.with_env(|mut env| {
            _ = env.call_method(self.controller.as_obj(), "play", "()V", &[])?;
            Ok(())
        })
    }

    async fn pause(&self) -> Result<(), anyhow::Error> {
        self.with_env(|mut env| {
            _ = env.call_method(self.controller.as_obj(), "pause", "()V", &[])?;
            Ok(())
        })
    }

    async fn volume(&self) -> Result<f64, anyhow::Error> {
        self.with_env(|mut env| {
            Ok(env
                .call_method(self.controller.as_obj(), "getVolume", "()F", &[])?
                .f()? as f64)
        })
    }

    async fn set_volume(&self, value: f64) -> Result<(), anyhow::Error> {
        self.with_env(|mut env| {
            env.call_method(
                self.controller.as_obj(),
                "setVolume",
                "(F)V",
                &[(value as f32).into()],
            )?;
            Ok(())
        })
    }

    async fn add_media_items(&self, items: Vec<MediaItem>) -> Result<(), anyhow::Error> {
        self.with_env(|mut env| {
            let vm = self.ctx.jvm().context("get java vm")?;
            let mut env = vm.get_env()?;

            let array =
                env.new_object_array(items.len() as i32, "java/lang/String", JObject::null())?;

            for (i, item) in items.into_iter().enumerate() {
                env.set_object_array_element(&array, i as i32, env.new_string(item.uri)?)?;
            }

            env.call_method(
                self.controller.as_obj(),
                "addMediaItems",
                "([Ljava/lang/String;)V",
                &[JValue::Object(&array)],
            )?;
            Ok(())
        })
    }

    async fn listen(&self) -> Result<PlayerEventStream, anyhow::Error> {
        todo!()
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

            let callback_class = find_class(
                &mut env,
                &activity,
                "org/yydcnjjw/mtool/assistant/Callback".into(),
            )?;

            let (tx, rx) = oneshot::channel();
            let sender = Box::into_raw(Box::new(tx)) as i64;

            let callback = env.new_object(callback_class, "(J)V", &[JValue::Long(sender)])?;

            env.call_static_method(
                controller_class,
                "connect",
                "(Landroid/content/Context;Lorg/yydcnjjw/mtool/assistant/Callback;)V",
                &[JValue::Object(&activity), JValue::Object(&callback)],
            )?;
            rx
        };

        let controller = rx.await?;

        info!("MediaPlayer is ready");

        Ok(Self { ctx, controller })
    }

    fn with_env<F, O>(&self, f: F) -> Result<O, anyhow::Error>
    where
        F: for<'local> FnOnce(JNIEnv<'local>) -> Result<O, anyhow::Error>,
    {
        let vm = self.ctx.jvm().context("get java vm")?;
        let env = vm.get_env()?;
        f(env)
    }
}

android_fn![
    org_yydcnjjw_mtool,
    assistant,
    Callback,
    invokeNative,
    [i64, JObject<'local>]
];

#[allow(non_snake_case)]
unsafe fn invokeNative<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    controller: JObject<'local>,
) {
    _ = Box::from_raw(handle as *mut oneshot::Sender<GlobalRef>)
        .send(env.new_global_ref(controller).unwrap());
}

android_fn![
    org_yydcnjjw_mtool,
    assistant,
    PlaybackService,
    realUri,
    [JString<'local>],
    JString<'local>
];

#[allow(non_snake_case)]
unsafe fn realUri<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    uri: JString<'local>,
) -> JString<'local> {
    env.get_string(&uri)
        .context("parse uri")
        .and_then(|uri| {
            let uri = uri.to_string_lossy().to_string();
            info!("parse {uri}");
            Netease::new()
                .get_song(uri)
                .and_then(|uri| env.new_string(uri.url).context("new string"))
        })
        .inspect_err(|e| {
            warn!("{e:?}");
        })
        .unwrap_or(JString::default())
}
