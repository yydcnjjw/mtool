use std::{any::type_name, marker::PhantomData};

use mapp::{
    anyhow::{self, Context},
    prelude::*,
};
use mtool_dioxus::desktop::wry::prelude::{
    android_fn,
    jni::{
        objects::{JObjectArray, JValue},
        sys::jlong,
    },
    *,
};

use super::provider::{ArgsProvide, JavaArgs};

android_fn![org_yydcnjjw_mtool, assistant, OnceCallback, invokeNative, [i64, JObjectArray<'local>], JObject<'local>];

#[allow(non_snake_case)]
unsafe fn invokeNative<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    args: JObjectArray<'local>,
) -> JObject<'local> {
    unsafe { Box::from_raw(handle as *mut DynOnceCallback) }
        .inner
        .invoke(env, args)
}

struct CallbackWrapper<Func, Args, Output> {
    f: Func,
    _phantom_data: PhantomData<(Args, Output)>,
}

impl<Func, Args> OnceCallback for CallbackWrapper<Func, Args, ()>
where
    Func: InjectOnce<Args, Output = ()>,
    Args: for<'a> ArgsProvide<JavaArgs<'a>>,
{
    fn invoke<'local>(
        self: Box<Self>,
        mut env: JNIEnv<'local>,
        args: JObjectArray<'local>,
    ) -> JObject<'local> {
        if let Err(e) = inject_once(
            &mut JavaArgs {
                env: unsafe { env.unsafe_clone() },
                args,
            },
            self.f,
        ) {
            env.throw_new("java/lang/RuntimeException", format!("{e:?}"))
                .unwrap();
        }
        JObject::null()
    }
}

fn inject_once<Func, Args, Output, C>(c: &mut C, f: Func) -> Result<Output, anyhow::Error>
where
    Func: InjectOnce<Args, Output = Output>,
    Args: ArgsProvide<C>,
{
    Ok(f.inject_once(
        Args::args_provide(c).context(format!("Failed to inject once {}", type_name::<Args>()))?,
    ))
}

pub trait OnceCallback {
    fn invoke<'local>(
        self: Box<Self>,
        env: JNIEnv<'local>,
        args: JObjectArray<'local>,
    ) -> JObject<'local>;
}

pub struct DynOnceCallback {
    inner: Box<dyn OnceCallback>,
}

impl DynOnceCallback {
    pub fn invoke<'local>(
        self,
        env: JNIEnv<'local>,
        args: JObjectArray<'local>,
    ) -> JObject<'local> {
        self.inner.invoke(env, args)
    }
}

pub fn new_once_callback<'local, Func, Args, Output>(
    env: &mut JNIEnv<'local>,
    activity: &JObject<'local>,
    f: Func,
) -> Result<JObject<'local>, anyhow::Error>
where
    Func: InjectOnce<Args, Output = Output> + 'static,
    Args: for<'a> ArgsProvide<JavaArgs<'local>> + 'static,
    Output: 'static,
    CallbackWrapper<Func, Args, Output>: OnceCallback,
{
    let this = Box::into_raw(Box::new(DynOnceCallback {
        inner: Box::new(CallbackWrapper::<Func, Args, Output> {
            f,
            _phantom_data: Default::default(),
        }),
    })) as i64;

    let callback_class = find_class(
        env,
        &activity,
        "org/yydcnjjw/mtool/assistant/OnceCallback".into(),
    )?;

    Ok(env.new_object(callback_class, "(J)V", &[JValue::Long(this)])?)
}
