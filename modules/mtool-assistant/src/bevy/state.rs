use std::sync::Arc;

use bevy::prelude::*;
use mapp::sync::Mutex;

pub struct BevyStateInner {
    id: Option<Entity>,
}

#[derive(Resource, Clone)]
pub struct BevyState {
    inner: Arc<Mutex<BevyStateInner>>,
}

impl BevyState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(BevyStateInner { id: None })),
        }
    }

    pub fn set_window_id(&self, id: Entity) {
        self.inner.lock().id = Some(id);
    }

    pub fn window_id(&self) -> Option<Entity> {
        self.inner.lock().id
    }

    pub fn window_id_uncheck(&self) -> Entity {
        self.inner.lock().id.unwrap()
    }    
}
