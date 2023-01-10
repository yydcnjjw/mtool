use std::{cell::RefCell, future::Future, rc::Rc};

use anyhow::Context;
use async_trait::async_trait;
use mkeybinding::{KeyCombine, KeyDispatcher};
use wasm_bindgen_futures::spawn_local;

#[async_trait(?Send)]
pub trait Action {
    async fn do_action(&mut self) -> Result<(), anyhow::Error>;
}

#[async_trait(?Send)]
impl<Func, Output> Action for Func
where
    Func: FnMut() -> Output,
    Output: Future<Output = Result<(), anyhow::Error>>,
{
    async fn do_action(&mut self) -> Result<(), anyhow::Error> {
        (self)().await
    }
}

type SharedAction = Rc<RefCell<dyn Action>>;

type Dispatcher = KeyDispatcher<SharedAction>;

#[derive(Clone)]
pub struct Keybinging {
    dispatcher: Rc<RefCell<Dispatcher>>,
}

impl PartialEq for Keybinging {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Keybinging {
    pub fn new() -> Self {
        let dispatcher = Rc::new(RefCell::new(KeyDispatcher::new()));

        Self { dispatcher }
    }

    pub async fn run_loop(&self) {
        let mut rx = self.dispatcher.borrow().subscribe();

        while let Ok(action) = rx.recv().await {
            spawn_local(async move {
                action.borrow_mut().do_action().await.unwrap();
            });
        }
    }

    pub fn dispatch(&self, key: KeyCombine) -> bool {
        self.dispatcher.borrow_mut().dispatch(key)
    }

    pub fn define<T>(&self, kbd: &str, action: T) -> Result<(), anyhow::Error>
    where
        T: Action + 'static,
    {
        self.define_raw(kbd, Rc::new(RefCell::new(action)))
    }

    fn define_raw(&self, kbd: &str, action: SharedAction) -> Result<(), anyhow::Error> {
        self.dispatcher
            .borrow_mut()
            .keymap()
            .add(kbd, action)
            .context(format!("Failed to define key binding {}", kbd))
    }

    pub fn remove(&self, kbd: &str) -> Result<(), anyhow::Error> {
        self.dispatcher
            .borrow_mut()
            .keymap()
            .remove(kbd)
            .context(format!("Failed to remove key binding {}", kbd))
    }
}
