use std::{cell::RefCell, future::Future, rc::Rc};

use mapp::{
    anyhow,
    js_sys::Function,
    prelude::*,
    wasm_bindgen::{closure::Closure, JsCast},
};
use msysev::prelude::*;
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
                let keyev = into_key_event(e.clone(), KeyState::Press);
                if let PhysicalKey::Code(key) = keyev.key {
                    if keybinding.dispatch(KeyCombine {
                        key,
                        mods: keyev.modifiers,
                    }) {
                        e.prevent_default();
                    }
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

fn from_key_code_attribute_value(kcav: &str) -> PhysicalKey {
    PhysicalKey::Code(match kcav {
        "Backquote" => KeyCode::Backquote,
        "Backslash" => KeyCode::Backslash,
        "BracketLeft" => KeyCode::BracketLeft,
        "BracketRight" => KeyCode::BracketRight,
        "Comma" => KeyCode::Comma,
        "Digit0" => KeyCode::Digit0,
        "Digit1" => KeyCode::Digit1,
        "Digit2" => KeyCode::Digit2,
        "Digit3" => KeyCode::Digit3,
        "Digit4" => KeyCode::Digit4,
        "Digit5" => KeyCode::Digit5,
        "Digit6" => KeyCode::Digit6,
        "Digit7" => KeyCode::Digit7,
        "Digit8" => KeyCode::Digit8,
        "Digit9" => KeyCode::Digit9,
        "Equal" => KeyCode::Equal,
        "IntlBackslash" => KeyCode::IntlBackslash,
        "IntlRo" => KeyCode::IntlRo,
        "IntlYen" => KeyCode::IntlYen,
        "KeyA" => KeyCode::KeyA,
        "KeyB" => KeyCode::KeyB,
        "KeyC" => KeyCode::KeyC,
        "KeyD" => KeyCode::KeyD,
        "KeyE" => KeyCode::KeyE,
        "KeyF" => KeyCode::KeyF,
        "KeyG" => KeyCode::KeyG,
        "KeyH" => KeyCode::KeyH,
        "KeyI" => KeyCode::KeyI,
        "KeyJ" => KeyCode::KeyJ,
        "KeyK" => KeyCode::KeyK,
        "KeyL" => KeyCode::KeyL,
        "KeyM" => KeyCode::KeyM,
        "KeyN" => KeyCode::KeyN,
        "KeyO" => KeyCode::KeyO,
        "KeyP" => KeyCode::KeyP,
        "KeyQ" => KeyCode::KeyQ,
        "KeyR" => KeyCode::KeyR,
        "KeyS" => KeyCode::KeyS,
        "KeyT" => KeyCode::KeyT,
        "KeyU" => KeyCode::KeyU,
        "KeyV" => KeyCode::KeyV,
        "KeyW" => KeyCode::KeyW,
        "KeyX" => KeyCode::KeyX,
        "KeyY" => KeyCode::KeyY,
        "KeyZ" => KeyCode::KeyZ,
        "Minus" => KeyCode::Minus,
        "Period" => KeyCode::Period,
        "Quote" => KeyCode::Quote,
        "Semicolon" => KeyCode::Semicolon,
        "Slash" => KeyCode::Slash,
        "AltLeft" => KeyCode::AltLeft,
        "AltRight" => KeyCode::AltRight,
        "Backspace" => KeyCode::Backspace,
        "CapsLock" => KeyCode::CapsLock,
        "ContextMenu" => KeyCode::ContextMenu,
        "ControlLeft" => KeyCode::ControlLeft,
        "ControlRight" => KeyCode::ControlRight,
        "Enter" => KeyCode::Enter,
        "MetaLeft" => KeyCode::SuperLeft,
        "MetaRight" => KeyCode::SuperRight,
        "ShiftLeft" => KeyCode::ShiftLeft,
        "ShiftRight" => KeyCode::ShiftRight,
        "Space" => KeyCode::Space,
        "Tab" => KeyCode::Tab,
        "Convert" => KeyCode::Convert,
        "KanaMode" => KeyCode::KanaMode,
        "Lang1" => KeyCode::Lang1,
        "Lang2" => KeyCode::Lang2,
        "Lang3" => KeyCode::Lang3,
        "Lang4" => KeyCode::Lang4,
        "Lang5" => KeyCode::Lang5,
        "NonConvert" => KeyCode::NonConvert,
        "Delete" => KeyCode::Delete,
        "End" => KeyCode::End,
        "Help" => KeyCode::Help,
        "Home" => KeyCode::Home,
        "Insert" => KeyCode::Insert,
        "PageDown" => KeyCode::PageDown,
        "PageUp" => KeyCode::PageUp,
        "ArrowDown" => KeyCode::ArrowDown,
        "ArrowLeft" => KeyCode::ArrowLeft,
        "ArrowRight" => KeyCode::ArrowRight,
        "ArrowUp" => KeyCode::ArrowUp,
        "NumLock" => KeyCode::NumLock,
        "Numpad0" => KeyCode::Numpad0,
        "Numpad1" => KeyCode::Numpad1,
        "Numpad2" => KeyCode::Numpad2,
        "Numpad3" => KeyCode::Numpad3,
        "Numpad4" => KeyCode::Numpad4,
        "Numpad5" => KeyCode::Numpad5,
        "Numpad6" => KeyCode::Numpad6,
        "Numpad7" => KeyCode::Numpad7,
        "Numpad8" => KeyCode::Numpad8,
        "Numpad9" => KeyCode::Numpad9,
        "NumpadAdd" => KeyCode::NumpadAdd,
        "NumpadBackspace" => KeyCode::NumpadBackspace,
        "NumpadClear" => KeyCode::NumpadClear,
        "NumpadClearEntry" => KeyCode::NumpadClearEntry,
        "NumpadComma" => KeyCode::NumpadComma,
        "NumpadDecimal" => KeyCode::NumpadDecimal,
        "NumpadDivide" => KeyCode::NumpadDivide,
        "NumpadEnter" => KeyCode::NumpadEnter,
        "NumpadEqual" => KeyCode::NumpadEqual,
        "NumpadHash" => KeyCode::NumpadHash,
        "NumpadMemoryAdd" => KeyCode::NumpadMemoryAdd,
        "NumpadMemoryClear" => KeyCode::NumpadMemoryClear,
        "NumpadMemoryRecall" => KeyCode::NumpadMemoryRecall,
        "NumpadMemoryStore" => KeyCode::NumpadMemoryStore,
        "NumpadMemorySubtract" => KeyCode::NumpadMemorySubtract,
        "NumpadMultiply" => KeyCode::NumpadMultiply,
        "NumpadParenLeft" => KeyCode::NumpadParenLeft,
        "NumpadParenRight" => KeyCode::NumpadParenRight,
        "NumpadStar" => KeyCode::NumpadStar,
        "NumpadSubtract" => KeyCode::NumpadSubtract,
        "Escape" => KeyCode::Escape,
        "Fn" => KeyCode::Fn,
        "FnLock" => KeyCode::FnLock,
        "PrintScreen" => KeyCode::PrintScreen,
        "ScrollLock" => KeyCode::ScrollLock,
        "Pause" => KeyCode::Pause,
        "BrowserBack" => KeyCode::BrowserBack,
        "BrowserFavorites" => KeyCode::BrowserFavorites,
        "BrowserForward" => KeyCode::BrowserForward,
        "BrowserHome" => KeyCode::BrowserHome,
        "BrowserRefresh" => KeyCode::BrowserRefresh,
        "BrowserSearch" => KeyCode::BrowserSearch,
        "BrowserStop" => KeyCode::BrowserStop,
        "Eject" => KeyCode::Eject,
        "LaunchApp1" => KeyCode::LaunchApp1,
        "LaunchApp2" => KeyCode::LaunchApp2,
        "LaunchMail" => KeyCode::LaunchMail,
        "MediaPlayPause" => KeyCode::MediaPlayPause,
        "MediaSelect" => KeyCode::MediaSelect,
        "MediaStop" => KeyCode::MediaStop,
        "MediaTrackNext" => KeyCode::MediaTrackNext,
        "MediaTrackPrevious" => KeyCode::MediaTrackPrevious,
        "Power" => KeyCode::Power,
        "Sleep" => KeyCode::Sleep,
        "AudioVolumeDown" => KeyCode::AudioVolumeDown,
        "AudioVolumeMute" => KeyCode::AudioVolumeMute,
        "AudioVolumeUp" => KeyCode::AudioVolumeUp,
        "WakeUp" => KeyCode::WakeUp,
        "Hyper" => KeyCode::Hyper,
        "Turbo" => KeyCode::Turbo,
        "Abort" => KeyCode::Abort,
        "Resume" => KeyCode::Resume,
        "Suspend" => KeyCode::Suspend,
        "Again" => KeyCode::Again,
        "Copy" => KeyCode::Copy,
        "Cut" => KeyCode::Cut,
        "Find" => KeyCode::Find,
        "Open" => KeyCode::Open,
        "Paste" => KeyCode::Paste,
        "Props" => KeyCode::Props,
        "Select" => KeyCode::Select,
        "Undo" => KeyCode::Undo,
        "Hiragana" => KeyCode::Hiragana,
        "Katakana" => KeyCode::Katakana,
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
        "F13" => KeyCode::F13,
        "F14" => KeyCode::F14,
        "F15" => KeyCode::F15,
        "F16" => KeyCode::F16,
        "F17" => KeyCode::F17,
        "F18" => KeyCode::F18,
        "F19" => KeyCode::F19,
        "F20" => KeyCode::F20,
        "F21" => KeyCode::F21,
        "F22" => KeyCode::F22,
        "F23" => KeyCode::F23,
        "F24" => KeyCode::F24,
        "F25" => KeyCode::F25,
        "F26" => KeyCode::F26,
        "F27" => KeyCode::F27,
        "F28" => KeyCode::F28,
        "F29" => KeyCode::F29,
        "F30" => KeyCode::F30,
        "F31" => KeyCode::F31,
        "F32" => KeyCode::F32,
        "F33" => KeyCode::F33,
        "F34" => KeyCode::F34,
        "F35" => KeyCode::F35,
        _ => return PhysicalKey::Unidentified(NativeKeyCode::Unidentified),
    })
}

pub fn into_key_event(e: KeyboardEvent, state: KeyState) -> KeyEvent {
    let key = from_key_code_attribute_value(e.code().as_str());

    let mut modifiers = ModifierState::NONE;
    if e.get_modifier_state("Alt") {
        modifiers |= ModifierState::ALT;
    }

    if e.get_modifier_state("Shift") {
        modifiers |= ModifierState::SHIFT;
    }

    if e.get_modifier_state("Control") {
        modifiers |= ModifierState::CONTROL;
    }

    if e.get_modifier_state("Meta") {
        modifiers |= ModifierState::SUPER;
    }

    if e.get_modifier_state("NumLock") {
        modifiers |= ModifierState::NUMLOCK;
    }

    if e.get_modifier_state("CapsLock") {
        modifiers |= ModifierState::CAPSLOCK;
    }

    KeyEvent {
        key,
        modifiers,
        state,
    }
}
