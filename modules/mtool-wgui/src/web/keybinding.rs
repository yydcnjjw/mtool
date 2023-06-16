use std::{cell::RefCell, future::Future, rc::Rc};

use async_trait::async_trait;
use js_sys::Function;
use msysev::{
    keydef::{KeyCode, KeyModifier},
    KeyAction, KeyEvent,
};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, KeyboardEvent};
use yew::platform::spawn_local;

pub use mkeybinding::*;

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
pub struct Keybinding {
    dispatcher: Rc<RefCell<Dispatcher>>,
}

impl PartialEq for Keybinding {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

#[macro_export]
macro_rules! generate_keymap {
    ($(($kbd:expr, $action:expr),)+) => {
        $crate::KeyMap::<$crate::SharedAction>::new_with_vec(
            vec![
                $(
                    (
                        $kbd,
                        ::std::rc::Rc::new(::std::cell::RefCell::new($action)) as $crate::SharedAction
                    )
                ),+
            ]
        )
    };
}

impl Keybinding {
    pub fn new_with_window() -> Self {
        let keybinding = Keybinding::new();

        keybinding.setup_on_keydown(|f| {
            window().unwrap().set_onkeydown(f);
        });

        keybinding
    }

    pub fn new() -> Self {
        let dispatcher = Rc::new(RefCell::new(KeyDispatcher::new()));

        Self { dispatcher }
    }

    pub fn setup_on_keydown<F>(&self, f: F)
    where
        F: Fn(Option<&Function>),
    {
        {
            let keybinding = self.clone();
            let a = Closure::<dyn FnMut(_)>::new(move |e: KeyboardEvent| {
                let keyev = into_key_event(e.clone(), KeyAction::Press);
                if keybinding.dispatch(KeyCombine {
                    key: keyev.keycode,
                    mods: keyev.modifiers,
                }) {
                    e.prevent_default();
                }
            });

            f(Some(a.as_ref().unchecked_ref()));

            a.forget();
        }

        {
            let keybinding = self.clone();
            spawn_local(async move {
                keybinding.run_loop().await;
            });
        }
    }

    async fn run_loop(&self) {
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

pub fn into_key_event(e: KeyboardEvent, action: KeyAction) -> KeyEvent {
    let scancode = e.key_code();

    let keycode = match e.code().as_str() {
        "Backquote" => KeyCode::GraveAccent,
        "Digit1" => KeyCode::Num1,
        "Digit2" => KeyCode::Num2,
        "Digit3" => KeyCode::Num3,
        "Digit4" => KeyCode::Num4,
        "Digit5" => KeyCode::Num5,
        "Digit6" => KeyCode::Num6,
        "Digit7" => KeyCode::Num7,
        "Digit8" => KeyCode::Num8,
        "Digit9" => KeyCode::Num9,
        "Digit0" => KeyCode::Num0,
        "Minus" => KeyCode::Minus, // -
        "Equal" => KeyCode::Equal, // =
        "Backspace" => KeyCode::BackSpace,
        "Tab" => KeyCode::Tab,
        "KeyQ" => KeyCode::Q,
        "KeyW" => KeyCode::W,
        "KeyE" => KeyCode::E,
        "KeyR" => KeyCode::R,
        "KeyT" => KeyCode::T,
        "KeyY" => KeyCode::Y,
        "KeyU" => KeyCode::U,
        "KeyI" => KeyCode::I,
        "KeyO" => KeyCode::O,
        "KeyP" => KeyCode::P,
        "BracketLeft" => KeyCode::BracketLeft,   // [
        "BracketRight" => KeyCode::BracketRight, // ]
        "Backslash" => KeyCode::Backslash,       // \
        "CapsLock" => KeyCode::CapsLock,
        "KeyA" => KeyCode::A,
        "KeyS" => KeyCode::S,
        "KeyD" => KeyCode::D,
        "KeyF" => KeyCode::F,
        "KeyG" => KeyCode::G,
        "KeyH" => KeyCode::H,
        "KeyJ" => KeyCode::J,
        "KeyK" => KeyCode::K,
        "KeyL" => KeyCode::L,
        "Semicolon" => KeyCode::Semicolon, // ;
        "Quoto" => KeyCode::Apostrophe,    // '
        "Enter" => KeyCode::Return,
        "ShiftLeft" => KeyCode::LeftShift,
        "KeyZ" => KeyCode::Z,
        "KeyX" => KeyCode::X,
        "KeyC" => KeyCode::C,
        "KeyV" => KeyCode::V,
        "KeyB" => KeyCode::B,
        "KeyN" => KeyCode::N,
        "KeyM" => KeyCode::M,
        "Comma" => KeyCode::Comma,   // ,
        "Period" => KeyCode::Period, // .
        "Slash" => KeyCode::Slash,   // /
        "ShiftRight" => KeyCode::RightShift,
        "ControlLeft" => KeyCode::LeftControl,
        "AltLeft" => KeyCode::LeftAlt,
        "Space" => KeyCode::Spacebar,
        "AltRight" => KeyCode::RightAlt,
        "ControlRight" => KeyCode::RightControl,
        "Insert" => KeyCode::Insert,
        "Delete" => KeyCode::Delete,
        "ArrowLeft" => KeyCode::LeftArrow,
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        "ArrowUp" => KeyCode::UpArrow,
        "ArrowDown" => KeyCode::DownArrow,
        "PageUp" => KeyCode::PageUp,
        "PageDown" => KeyCode::PageDown,
        "ArrowRight" => KeyCode::RightArrow,
        "NumLock" => KeyCode::NumLock,
        "Numpad7" => KeyCode::Keypad7,
        "Numpad4" => KeyCode::Keypad4,
        "Numpad1" => KeyCode::Keypad1,
        "NumpadDivide" => KeyCode::Divide, // /
        "Numpad8" => KeyCode::Keypad8,
        "Numpad5" => KeyCode::Keypad5,
        "Numpad2" => KeyCode::Keypad2,
        "Numpad0" => KeyCode::Keypad0,
        "NumpadMultiply" => KeyCode::Multiply,
        "Numpad9" => KeyCode::Keypad9,
        "Numpad6" => KeyCode::Keypad6,
        "Numpad3" => KeyCode::Keypad3,
        "NumpadPeriod" => KeyCode::KeypadPeriod,
        "NumpadSubtract" => KeyCode::Subtract,
        "NumpadAdd" => KeyCode::Add,
        "NumpadComma" => KeyCode::KeypadComma,
        "NumpadEnter" => KeyCode::KeypadEnter,
        "Escape" => KeyCode::Escape,
        "F1" => KeyCode::F1,
        "F2" => KeyCode::F2,
        "F3" => KeyCode::F3,
        "F4" => KeyCode::F4,
        "F5" => KeyCode::F5,
        "F6" => KeyCode::F6,
        "F7" => KeyCode::F7,
        "F8" => KeyCode::F8,
        "F9" => KeyCode::F9,
        "F10" => KeyCode::F10,
        "F11" => KeyCode::F11,
        "F12" => KeyCode::F12,
        "PrintScreen" => KeyCode::PrintScreen,
        "ScrollLock" => KeyCode::ScrollLock,
        "Pause" => KeyCode::Pause,
        "MetaLeft" => KeyCode::LeftGUI,        // super_L
        "MetaRight" => KeyCode::RightGUI,      // super_R
        "ContextMenu" => KeyCode::Application, // menu

        _ => KeyCode::Unknown,
    };

    let mut modifiers = KeyModifier::NONE;
    if e.get_modifier_state("Alt") {
        modifiers |= KeyModifier::ALT;
    }

    if e.get_modifier_state("Shift") {
        modifiers |= KeyModifier::SHIFT;
    }

    if e.get_modifier_state("Control") {
        modifiers |= KeyModifier::CONTROL;
    }

    if e.get_modifier_state("Meta") {
        modifiers |= KeyModifier::SUPER;
    }

    if e.get_modifier_state("NumLock") {
        modifiers |= KeyModifier::NUMLOCK;
    }

    if e.get_modifier_state("CapsLock") {
        modifiers |= KeyModifier::CAPSLOCK;
    }

    KeyEvent {
        scancode,
        keycode,
        modifiers,
        action,
    }
}
