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

android_fn![org_yydcnjjw_mtool, assistant, Callback, invokeNative, [i64, JObjectArray<'local>], JObject<'local>];

#[allow(non_snake_case)]
unsafe fn invokeNative<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    args: JObjectArray<'local>,
) -> JObject<'local> {
    (&mut *(handle as *mut DynCallback)).invoke(env, args)
}

struct CallbackWrapper<Func, Args, Output> {
    f: Func,
    _phantom_data: PhantomData<(Args, Output)>,
}

impl<Func, Args> Callback for CallbackWrapper<Func, Args, ()>
where
    Func: Inject<Args, Output = ()>,
    Args: for<'a> ArgsProvide<JavaArgs<'a>>,
{
    fn invoke<'local>(
        &mut self,
        mut env: JNIEnv<'local>,
        args: JObjectArray<'local>,
    ) -> JObject<'local> {
        if let Err(e) = inject(
            &mut JavaArgs {
                env: unsafe { env.unsafe_clone() },
                args,
            },
            &self.f,
        ) {
            env.throw_new("java/lang/RuntimeException", format!("{e:?}"))
                .unwrap();
        }
        JObject::null()
    }
}

fn inject<Func, Args, Output, C>(c: &mut C, f: &Func) -> Result<Output, anyhow::Error>
where
    Func: Inject<Args, Output = Output>,
    Args: ArgsProvide<C>,
{
    Ok(f.inject(
        Args::args_provide(c).context(format!("Failed to inject {}", type_name::<Args>()))?,
    ))
}

pub trait Callback {
    fn invoke<'local>(
        &mut self,
        env: JNIEnv<'local>,
        args: JObjectArray<'local>,
    ) -> JObject<'local>;
}

pub struct DynCallback {
    inner: Box<dyn Callback>,
}

impl DynCallback {
    pub fn invoke<'local>(
        &mut self,
        env: JNIEnv<'local>,
        args: JObjectArray<'local>,
    ) -> JObject<'local> {
        self.inner.invoke(env, args)
    }
}

pub fn new_callback<'local, Func, Args, Output>(
    env: &mut JNIEnv<'local>,
    activity: &JObject<'local>,
    f: Func,
) -> Result<JObject<'local>, anyhow::Error>
where
    Func: Inject<Args, Output = Output> + 'static,
    Args: for<'a> ArgsProvide<JavaArgs<'local>> + 'static,
    Output: 'static,
    CallbackWrapper<Func, Args, Output>: Callback,
{
    let this = Box::into_raw(Box::new(DynCallback {
        inner: Box::new(CallbackWrapper::<Func, Args, Output> {
            f,
            _phantom_data: Default::default(),
        }),
    })) as i64;

    let callback_class = find_class(
        env,
        &activity,
        "org/yydcnjjw/mtool/assistant/Callback".into(),
    )?;

    Ok(env.new_object(callback_class, "(J)V", &[JValue::Long(this)])?)
}
