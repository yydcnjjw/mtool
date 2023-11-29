use std::{
    any::{Any, TypeId},
    sync::Arc,
};

use dashmap::DashMap;
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

    fn bind<T>(&self, win: &tauri::Window, v: T)
    where
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

    fn get<T>(&self, win: &tauri::Window) -> Option<T>
    where
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

fn get_data_binding(win: &tauri::Window) -> State<'_, DataBinding> {
    if win.try_state::<DataBinding>().is_none() {
        win.manage(DataBinding::new());
    }

    win.state::<DataBinding>()
}

impl WindowDataBind for tauri::Window {
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
        binding.get::<T>(self)
    }
}
