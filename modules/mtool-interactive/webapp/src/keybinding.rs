use std::{cell::RefCell, future::Future, rc::Rc};

use async_trait::async_trait;
use mkeybinding::{KeyCombine, KeyDispatcher, KeyMap};
use msysev::KeyAction;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, KeyboardEvent};

use crate::event::into_key_event;

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

pub type SharedAction = Rc<RefCell<dyn Action>>;

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

#[macro_export]
macro_rules! generate_keymap {
    ($(($kbd:expr, $action:expr),)+) => {
        ::mkeybinding::KeyMap::<$crate::keybinding::SharedAction>::new_with_vec(
            vec![
                $(
                    (
                        $kbd,
                        ::std::rc::Rc::new(::std::cell::RefCell::new($action)) as $crate::keybinding::SharedAction
                    )
                ),+
            ]
        )
    };
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

    pub fn push_keymap(&self, id: &str, km: KeyMap<SharedAction>) {
        self.dispatcher.borrow_mut().push_keymap(id, km);
    }

    pub fn pop_keymap(&self) -> Option<(String, KeyMap<SharedAction>)> {
        self.dispatcher.borrow_mut().pop_keymap()
    }

    pub fn contains_keymap(&self, id: &str) -> bool {
        self.dispatcher.borrow().contains_keymap(id)
    }

    pub fn remove_keymap(&self, id: &str) -> Option<(String, KeyMap<SharedAction>)> {
        self.dispatcher.borrow_mut().remove_keymap(id)
    }
}

pub fn setup() -> Keybinging {
    let keybinding = Keybinging::new();
    {
        let keybinding = keybinding.clone();
        let a = Closure::<dyn FnMut(_)>::new(move |e: KeyboardEvent| {
            let keyev = into_key_event(e.clone(), KeyAction::Press);
            if keybinding.dispatch(KeyCombine {
                key: keyev.keycode,
                mods: keyev.modifiers,
            }) {
                e.prevent_default();
            }
        });

        window()
            .unwrap()
            .set_onkeydown(Some(a.as_ref().unchecked_ref()));

        a.forget();
    }

    {
        let keybinding = keybinding.clone();
        spawn_local(async move {
            keybinding.run_loop().await;
        });
    }

    keybinding
}
