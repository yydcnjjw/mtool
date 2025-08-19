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

use crate::music::{Netease, Playlist};

#[derive(Clone)]
pub struct MediaPlayer {
    ctx: Res<DioxusContext>,
    controller: GlobalRef,
}

impl MediaPlayer {
    pub async fn new(ctx: Res<DioxusContext>) -> Result<Self, anyhow::Error> {
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

        let controller = rx.await?;

        info!("MediaPlayer is ready");

        Ok(Self { ctx, controller })
    }

    pub fn play(&self) -> Result<(), anyhow::Error> {
        let vm = self.ctx.jvm().context("get java vm")?;
        let mut env = vm.get_env()?;
        env.call_method(self.controller.as_obj(), "play", "()V", &[])?;
        Ok(())
    }

    pub fn pause(&self) -> Result<(), anyhow::Error> {
        let vm = self.ctx.jvm().context("get java vm")?;
        let mut env = vm.get_env()?;
        env.call_method(self.controller.as_obj(), "pause", "()V", &[])?;
        Ok(())
    }

    pub fn volume(&self) -> Result<usize, anyhow::Error> {
        let vm = self.ctx.jvm().context("get java vm")?;
        let mut env = vm.get_env()?;
        let volume = env
            .call_method(self.controller.as_obj(), "getVolume", "()F", &[])?
            .f()?;

        Ok((volume.clamp(0., 1.) * 100.).round() as usize)
    }

    pub fn set_volume(&mut self, value: usize) -> Result<(), anyhow::Error> {
        let vm = self.ctx.jvm().context("get java vm")?;
        let mut env = vm.get_env()?;
        env.call_method(
            self.controller.as_obj(),
            "setVolume",
            "(F)V",
            &[(value as f32 / 100.).into()],
        )?;

        Ok(())
    }

    pub fn add_playlist(&self, netease_playlist: Playlist) -> Result<(), anyhow::Error> {
        let playlist = netease_playlist
            .entries
            .into_iter()
            .map(|entry| entry.url)
            .collect_vec();
        let vm = self.ctx.jvm().context("get java vm")?;
        let mut env = vm.get_env()?;

        let array =
            env.new_object_array(playlist.len() as i32, "java/lang/String", JObject::null())?;

        for (i, item) in playlist.iter().enumerate() {
            env.set_object_array_element(&array, i as i32, env.new_string(item)?)?;
        }

        env.call_method(
            self.controller.as_obj(),
            "addPlaylist",
            "([Ljava/lang/String;)V",
            &[JValue::Object(&array)],
        )?;

        Ok(())
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
