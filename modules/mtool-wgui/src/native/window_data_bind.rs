use std::{
    any::{Any, TypeId},
    sync::Arc,
};

use mapp::dashmap::DashMap;
use tauri::{Manager, State, WindowEvent};

type BoxedAny = Box<dyn Any + Send + Sync>;
type AnyMap = DashMap<TypeId, BoxedAny>;

struct DataBinding {
    inner: Arc<DashMap<String, AnyMap>>,
}
impl DataBinding {
    fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
        }
    }

    fn bind<R, T>(&self, win: &tauri::WebviewWindow<R>, v: T)
    where
        R: tauri::Runtime,
        T: Send + Sync + Clone + 'static,
    {
        let label = win.label().to_string();

        self.inner
            .entry(label.clone())
            .or_insert_with(|| {
                let inner = self.inner.clone();
                win.on_window_event(move |e| {
                    if let WindowEvent::Destroyed = e {
                        inner.remove(&label);
                    }
                });

                DashMap::new()
            })
            .insert(TypeId::of::<T>(), Box::new(v));
    }

    fn get<R, T>(&self, win: &tauri::WebviewWindow<R>) -> Option<T>
    where
        R: tauri::Runtime,
        T: Send + Sync + Clone + 'static,
    {
        self.inner
            .get(win.label())?
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>().map(|v| v.clone()))
    }
}

pub trait WindowDataBind {
    fn bind<T>(&self, v: T)
    where
        T: Send + Sync + Clone + 'static;

    fn get_data<T>(&self) -> Option<T>
    where
        T: Send + Sync + Clone + 'static;
}

fn get_data_binding<R>(win: &tauri::WebviewWindow<R>) -> State<'_, DataBinding>
where
    R: tauri::Runtime,
{
    if win.try_state::<DataBinding>().is_none() {
        win.manage(DataBinding::new());
    }

    win.state::<DataBinding>()
}

impl<R> WindowDataBind for tauri::WebviewWindow<R>
where
    R: tauri::Runtime,
{
    fn bind<T>(&self, v: T)
    where
        T: Send + Sync + Clone + 'static,
    {
        let binding = get_data_binding(self);
        binding.bind(self, v);
    }

    fn get_data<T>(&self) -> Option<T>
    where
        T: Send + Sync + Clone + 'static,
    {
        let binding = get_data_binding(self);
        binding.get::<_, T>(self)
    }
}
