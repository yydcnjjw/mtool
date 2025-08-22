use mapp::{anyhow, prelude::*};
use mtool_dioxus::desktop::wry::prelude::{jni::objects::JObjectArray, *};

pub struct JavaArgs<'local> {
    pub env: JNIEnv<'local>,
    pub args: JObjectArray<'local>,
}

pub trait ArgsProvide<C>: Sized {
    fn args_provide(c: &mut C) -> Result<Self, anyhow::Error>;
}

macro_rules! impl_provider_for_tuple_with_container {
    ($((($param: ident), $index: expr)),*) => {
        impl<C, $($param,)*> ArgsProvide<C> for ($($param,)*)
        where
            $($param: for<'a> ArgsProvide<(&'a mut C, usize)>,)*
        {
            #[allow(unused_variables)]
            fn args_provide(c: &mut C) -> Result<Self, anyhow::Error> {
                Ok(($($param::args_provide(&mut (c, $index))?,)*))
            }
        }
    };
}

repeat!(
    9,
    enum_params_with_index,
    impl_provider_for_tuple_with_container,
    P
);

impl<'local> ArgsProvide<(&mut JavaArgs<'local>, usize)> for GlobalRef {
    fn args_provide(
        (JavaArgs { ref mut env, args }, index): &mut (&mut JavaArgs<'local>, usize),
    ) -> Result<Self, anyhow::Error> {
        let o = env.get_object_array_element(args, *index as i32)?;
        Ok(env.new_global_ref(o)?)
    }
}

// TODO: error context class name
impl<'local> ArgsProvide<(&mut JavaArgs<'local>, usize)> for String {
    fn args_provide(
        (JavaArgs { ref mut env, args }, index): &mut (&mut JavaArgs<'local>, usize),
    ) -> Result<Self, anyhow::Error> {
        let o = env.get_object_array_element(args, *index as i32)?;
        Ok(env
            .get_string(&JString::from(o))?
            .to_string_lossy()
            .to_string())
    }
}
