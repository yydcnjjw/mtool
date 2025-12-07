use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_desktop::{
    window,
    winit::event::{ElementState, RawKeyEvent},
};
use mapp::{
    keyboard_types::{KeyState, Modifiers},
    sync::Mutex,
    tokio::{self},
    tracing::{debug, warn},
};
use mkeybinding::*;

use super::{keyboard::from_winit_physical_key, Action};

type Dispatcher = KeyDispatcher<String, Action>;

#[derive(Clone)]
pub struct Keybinding {
    dispatcher: Arc<Mutex<Dispatcher>>,
    modifiers: Arc<Mutex<Modifiers>>,
}

impl PartialEq for Keybinding {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Keybinding {
    pub fn new() -> Self {
        Self {
            dispatcher: Arc::new(Mutex::new(KeyDispatcher::new())),
            modifiers: Arc::new(Mutex::new(Modifiers::empty())),
        }
    }

    pub async fn run_loop(&self) {
        let mut rx = self.dispatcher.lock().subscribe();

        while let Ok((ks, action)) = rx.recv().await {
            debug!("{} is triggered", ks);
            match action {
                Action::Local(action) => {
                    spawn(async move {
                        if let Err(e) = action.local_run_action().await {
                            warn!("{:?}", e);
                        }
                    });
                }
                Action::Shared(action) => {
                    tokio::spawn(async move {
                        if let Err(e) = action.run_action().await {
                            warn!("{:?}", e);
                        }
                    });
                }
            };
        }
    }

    pub fn handle_web_key_down(&self, ev: Event<KeyboardData>) {
        let mods = ev.data().modifiers();
        let code = ev.data().code();
        self.dispatcher.lock().dispatch(KeyCombine { code, mods });
    }

    pub fn handle_device_event(&self, ev: &RawKeyEvent) {
        let key = from_winit_physical_key(&ev.physical_key);
        let state = match ev.state {
            ElementState::Pressed => KeyState::Down,
            ElementState::Released => KeyState::Up,
        };

        let kc = if let Some(code) = key {
            KeyCombine {
                code,
                mods: self.update_modifier_state(&code, &state),
            }
        } else {
            return;
        };

        if let KeyState::Up = state {
            return;
        }

        window().id();

        info!(?kc, "{:?}", self as *const Self);

        self.dispatcher.lock().dispatch(kc);
    }

    fn update_modifier_state(&self, code: &Code, state: &KeyState) -> Modifiers {
        let modifer = match code {
            Code::ShiftLeft | Code::ShiftRight => Modifiers::SHIFT,
            Code::CapsLock => Modifiers::CAPS_LOCK,
            Code::ControlLeft | Code::ControlRight => Modifiers::CONTROL,
            Code::AltLeft | Code::AltRight => Modifiers::ALT,
            Code::NumLock => Modifiers::NUM_LOCK,
            Code::Super => Modifiers::SUPER,
            _ => Modifiers::empty(),
        };

        let mut modifiers = self.modifiers.lock();
        match state {
            KeyState::Down => *modifiers |= modifer,
            KeyState::Up => *modifiers -= modifer,
        };
        *modifiers
    }

    pub fn push_keymap<Id>(&self, id: Id, km: KeyMap<Action>)
    where
        Id: AsRef<str>,
    {
        let id = id.as_ref();
        debug!("push_keymap {}", id);
        self.dispatcher.lock().push_keymap(&id.to_string(), km);
    }

    pub fn contains_keymap<Id>(&self, id: Id) -> bool
    where
        Id: AsRef<str>,
    {
        let id = id.as_ref();
        self.dispatcher.lock().contains_keymap(&id.to_string())
    }

    pub fn remove_keymap<Id>(&self, id: Id) -> Option<(String, KeyMap<Action>)>
    where
        Id: AsRef<str>,
    {
        let id = id.as_ref();
        debug!("remove_keymap {}", id);
        self.dispatcher.lock().remove_keymap(&id.to_string())
    }
}
