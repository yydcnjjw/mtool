use std::{future::Future, sync::Arc};

use dioxus::prelude::*;
use dioxus_desktop::winit::{
    event::{ElementState, RawKeyEvent},
    keyboard::{KeyCode as WinitKeyCode, PhysicalKey as WinitPhysicalKey},
};
use mapp::{
    anyhow,
    keyboard_types::{Code as KeyCode, KeyState, Modifiers},
    prelude::*,
    send_wrapper::SendWrapper,
    sync::Mutex,
    tokio::{self, sync::RwLock},
    tracing::{debug, warn},
};
pub use mkeybinding::*;

#[async_trait]
pub trait RunAction {
    async fn run_action(&self) -> Result<(), anyhow::Error>;
}

#[async_trait]
impl<Func, Output> RunAction for Func
where
    Func: Fn() -> Output + Send + Sync + 'static,
    Output: Future<Output = Result<(), anyhow::Error>> + Send + 'static,
{
    async fn run_action(&self) -> Result<(), anyhow::Error> {
        (self)().await
    }
}

pub type SharedAction = Arc<dyn RunAction + Send + Sync + 'static>;

#[async_trait(?Send)]
pub trait LocalRunAction {
    async fn local_run_action(&self) -> Result<(), anyhow::Error>;
}

#[async_trait(?Send)]
impl<Func, Output> LocalRunAction for Func
where
    Func: Fn() -> Output,
    Output: Future<Output = Result<(), anyhow::Error>>,
{
    async fn local_run_action(&self) -> Result<(), anyhow::Error> {
        (self)().await
    }
}

#[async_trait(?Send)]
impl LocalRunAction for Callback<(), Result<(), anyhow::Error>> {
    async fn local_run_action(&self) -> Result<(), anyhow::Error> {
        self.call(())
    }
}

pub type LocalAction = Arc<SendWrapper<Box<dyn LocalRunAction + 'static>>>;

#[derive(Clone)]
pub enum Action {
    Local(LocalAction),
    Shared(SharedAction),
}

type Dispatcher = KeyDispatcher<Action>;

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

#[macro_export]
macro_rules! generate_keymap {
    ($(($kbd:expr, $action:expr),)+) => {
        $crate::prelude::KeyMap::<$crate::prelude::Action>::new_with_vec(
            vec![
                $(
                    (
                        $kbd,
                        $action
                    )
                ),+
            ]
        )
    };
}

#[macro_export]
macro_rules! local_action {
    ($action:expr) => {
        $crate::prelude::Action::Local(::std::sync::Arc::new(
            ::mapp::send_wrapper::SendWrapper::new(Box::new($action)),
        ))
    };
}

#[macro_export]
macro_rules! shared_action {
    ($action:expr) => {
        $crate::prelude::Action::Shared(::std::sync::Arc::new(Box::new($action)))
    };
}

#[macro_export]
macro_rules! __tail_ident {
    ($($deref:ident).*) => {
        $crate::__tail_ident![@ $($deref).*]
    };

    ($($deref:ident)* @ $head:ident $( . $tail:ident)+) => {
        $crate::__tail_ident![$($deref)* $head @ $($tail).+]
    };

    ($($deref:ident)* @ $last:ident) => {
        $last
    };
}

#[macro_export]
macro_rules! __call {
    ($fn:expr, $($es:ident),*) => {
        $fn($( $crate::__tail_ident![$es] ),*)
    };
}

#[macro_export]
macro_rules! local_action_fn {
    ($fn:expr, $($rest:tt)*) => {
        $crate::local_action!({
            to_owned![$($rest)*];
            move || {
                to_owned![$($rest)*];
                $crate::__call![$fn, $($rest)*]
            }
        })
    };
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

    pub fn push_keymap(&self, id: &str, km: KeyMap<Action>) {
        debug!("push_keymap {}", id);
        self.dispatcher.lock().push_keymap(id, km);
    }

    pub fn contains_keymap(&self, id: &str) -> bool {
        self.dispatcher.lock().contains_keymap(id)
    }

    pub fn remove_keymap(&self, id: &str) -> Option<(String, KeyMap<Action>)> {
        debug!("remove_keymap {}", id);
        self.dispatcher.lock().remove_keymap(id)
    }
}

fn from_winit_physical_key(key: &WinitPhysicalKey) -> Option<KeyCode> {
    match key {
        WinitPhysicalKey::Code(key_code) => Some(match key_code {
            WinitKeyCode::Backquote => KeyCode::Backquote,
            WinitKeyCode::Backslash => KeyCode::Backslash,
            WinitKeyCode::BracketLeft => KeyCode::BracketLeft,
            WinitKeyCode::BracketRight => KeyCode::BracketRight,
            WinitKeyCode::Comma => KeyCode::Comma,
            WinitKeyCode::Digit0 => KeyCode::Digit0,
            WinitKeyCode::Digit1 => KeyCode::Digit1,
            WinitKeyCode::Digit2 => KeyCode::Digit2,
            WinitKeyCode::Digit3 => KeyCode::Digit3,
            WinitKeyCode::Digit4 => KeyCode::Digit4,
            WinitKeyCode::Digit5 => KeyCode::Digit5,
            WinitKeyCode::Digit6 => KeyCode::Digit6,
            WinitKeyCode::Digit7 => KeyCode::Digit7,
            WinitKeyCode::Digit8 => KeyCode::Digit8,
            WinitKeyCode::Digit9 => KeyCode::Digit9,
            WinitKeyCode::Equal => KeyCode::Equal,
            WinitKeyCode::IntlBackslash => KeyCode::IntlBackslash,
            WinitKeyCode::IntlRo => KeyCode::IntlRo,
            WinitKeyCode::IntlYen => KeyCode::IntlYen,
            WinitKeyCode::KeyA => KeyCode::KeyA,
            WinitKeyCode::KeyB => KeyCode::KeyB,
            WinitKeyCode::KeyC => KeyCode::KeyC,
            WinitKeyCode::KeyD => KeyCode::KeyD,
            WinitKeyCode::KeyE => KeyCode::KeyE,
            WinitKeyCode::KeyF => KeyCode::KeyF,
            WinitKeyCode::KeyG => KeyCode::KeyG,
            WinitKeyCode::KeyH => KeyCode::KeyH,
            WinitKeyCode::KeyI => KeyCode::KeyI,
            WinitKeyCode::KeyJ => KeyCode::KeyJ,
            WinitKeyCode::KeyK => KeyCode::KeyK,
            WinitKeyCode::KeyL => KeyCode::KeyL,
            WinitKeyCode::KeyM => KeyCode::KeyM,
            WinitKeyCode::KeyN => KeyCode::KeyN,
            WinitKeyCode::KeyO => KeyCode::KeyO,
            WinitKeyCode::KeyP => KeyCode::KeyP,
            WinitKeyCode::KeyQ => KeyCode::KeyQ,
            WinitKeyCode::KeyR => KeyCode::KeyR,
            WinitKeyCode::KeyS => KeyCode::KeyS,
            WinitKeyCode::KeyT => KeyCode::KeyT,
            WinitKeyCode::KeyU => KeyCode::KeyU,
            WinitKeyCode::KeyV => KeyCode::KeyV,
            WinitKeyCode::KeyW => KeyCode::KeyW,
            WinitKeyCode::KeyX => KeyCode::KeyX,
            WinitKeyCode::KeyY => KeyCode::KeyY,
            WinitKeyCode::KeyZ => KeyCode::KeyZ,
            WinitKeyCode::Minus => KeyCode::Minus,
            WinitKeyCode::Period => KeyCode::Period,
            WinitKeyCode::Quote => KeyCode::Quote,
            WinitKeyCode::Semicolon => KeyCode::Semicolon,
            WinitKeyCode::Slash => KeyCode::Slash,
            WinitKeyCode::AltLeft => KeyCode::AltLeft,
            WinitKeyCode::AltRight => KeyCode::AltRight,
            WinitKeyCode::Backspace => KeyCode::Backspace,
            WinitKeyCode::CapsLock => KeyCode::CapsLock,
            WinitKeyCode::ContextMenu => KeyCode::ContextMenu,
            WinitKeyCode::ControlLeft => KeyCode::ControlLeft,
            WinitKeyCode::ControlRight => KeyCode::ControlRight,
            WinitKeyCode::Enter => KeyCode::Enter,
            WinitKeyCode::SuperLeft => KeyCode::MetaLeft,
            WinitKeyCode::SuperRight => KeyCode::MetaRight,
            WinitKeyCode::ShiftLeft => KeyCode::ShiftLeft,
            WinitKeyCode::ShiftRight => KeyCode::ShiftRight,
            WinitKeyCode::Space => KeyCode::Space,
            WinitKeyCode::Tab => KeyCode::Tab,
            WinitKeyCode::Convert => KeyCode::Convert,
            WinitKeyCode::KanaMode => KeyCode::KanaMode,
            WinitKeyCode::Lang1 => KeyCode::Lang1,
            WinitKeyCode::Lang2 => KeyCode::Lang2,
            WinitKeyCode::Lang3 => KeyCode::Lang3,
            WinitKeyCode::Lang4 => KeyCode::Lang4,
            WinitKeyCode::Lang5 => KeyCode::Lang5,
            WinitKeyCode::NonConvert => KeyCode::NonConvert,
            WinitKeyCode::Delete => KeyCode::Delete,
            WinitKeyCode::End => KeyCode::End,
            WinitKeyCode::Help => KeyCode::Help,
            WinitKeyCode::Home => KeyCode::Home,
            WinitKeyCode::Insert => KeyCode::Insert,
            WinitKeyCode::PageDown => KeyCode::PageDown,
            WinitKeyCode::PageUp => KeyCode::PageUp,
            WinitKeyCode::ArrowDown => KeyCode::ArrowDown,
            WinitKeyCode::ArrowLeft => KeyCode::ArrowLeft,
            WinitKeyCode::ArrowRight => KeyCode::ArrowRight,
            WinitKeyCode::ArrowUp => KeyCode::ArrowUp,
            WinitKeyCode::NumLock => KeyCode::NumLock,
            WinitKeyCode::Numpad0 => KeyCode::Numpad0,
            WinitKeyCode::Numpad1 => KeyCode::Numpad1,
            WinitKeyCode::Numpad2 => KeyCode::Numpad2,
            WinitKeyCode::Numpad3 => KeyCode::Numpad3,
            WinitKeyCode::Numpad4 => KeyCode::Numpad4,
            WinitKeyCode::Numpad5 => KeyCode::Numpad5,
            WinitKeyCode::Numpad6 => KeyCode::Numpad6,
            WinitKeyCode::Numpad7 => KeyCode::Numpad7,
            WinitKeyCode::Numpad8 => KeyCode::Numpad8,
            WinitKeyCode::Numpad9 => KeyCode::Numpad9,
            WinitKeyCode::NumpadAdd => KeyCode::NumpadAdd,
            WinitKeyCode::NumpadBackspace => KeyCode::NumpadBackspace,
            WinitKeyCode::NumpadClear => KeyCode::NumpadClear,
            WinitKeyCode::NumpadClearEntry => KeyCode::NumpadClearEntry,
            WinitKeyCode::NumpadComma => KeyCode::NumpadComma,
            WinitKeyCode::NumpadDecimal => KeyCode::NumpadDecimal,
            WinitKeyCode::NumpadDivide => KeyCode::NumpadDivide,
            WinitKeyCode::NumpadEnter => KeyCode::NumpadEnter,
            WinitKeyCode::NumpadEqual => KeyCode::NumpadEqual,
            WinitKeyCode::NumpadHash => KeyCode::NumpadHash,
            WinitKeyCode::NumpadMemoryAdd => KeyCode::NumpadMemoryAdd,
            WinitKeyCode::NumpadMemoryClear => KeyCode::NumpadMemoryClear,
            WinitKeyCode::NumpadMemoryRecall => KeyCode::NumpadMemoryRecall,
            WinitKeyCode::NumpadMemoryStore => KeyCode::NumpadMemoryStore,
            WinitKeyCode::NumpadMemorySubtract => KeyCode::NumpadMemorySubtract,
            WinitKeyCode::NumpadMultiply => KeyCode::NumpadMultiply,
            WinitKeyCode::NumpadParenLeft => KeyCode::NumpadParenLeft,
            WinitKeyCode::NumpadParenRight => KeyCode::NumpadParenRight,
            WinitKeyCode::NumpadStar => KeyCode::NumpadStar,
            WinitKeyCode::NumpadSubtract => KeyCode::NumpadSubtract,
            WinitKeyCode::Escape => KeyCode::Escape,
            WinitKeyCode::Fn => KeyCode::Fn,
            WinitKeyCode::FnLock => KeyCode::FnLock,
            WinitKeyCode::PrintScreen => KeyCode::PrintScreen,
            WinitKeyCode::ScrollLock => KeyCode::ScrollLock,
            WinitKeyCode::Pause => KeyCode::Pause,
            WinitKeyCode::BrowserBack => KeyCode::BrowserBack,
            WinitKeyCode::BrowserFavorites => KeyCode::BrowserFavorites,
            WinitKeyCode::BrowserForward => KeyCode::BrowserForward,
            WinitKeyCode::BrowserHome => KeyCode::BrowserHome,
            WinitKeyCode::BrowserRefresh => KeyCode::BrowserRefresh,
            WinitKeyCode::BrowserSearch => KeyCode::BrowserSearch,
            WinitKeyCode::BrowserStop => KeyCode::BrowserStop,
            WinitKeyCode::Eject => KeyCode::Eject,
            WinitKeyCode::LaunchApp1 => KeyCode::LaunchApp1,
            WinitKeyCode::LaunchApp2 => KeyCode::LaunchApp2,
            WinitKeyCode::LaunchMail => KeyCode::LaunchMail,
            WinitKeyCode::MediaPlayPause => KeyCode::MediaPlayPause,
            WinitKeyCode::MediaSelect => KeyCode::MediaSelect,
            WinitKeyCode::MediaStop => KeyCode::MediaStop,
            WinitKeyCode::MediaTrackNext => KeyCode::MediaTrackNext,
            WinitKeyCode::MediaTrackPrevious => KeyCode::MediaTrackPrevious,
            WinitKeyCode::Power => KeyCode::Power,
            WinitKeyCode::Sleep => KeyCode::Sleep,
            WinitKeyCode::AudioVolumeDown => KeyCode::AudioVolumeDown,
            WinitKeyCode::AudioVolumeMute => KeyCode::AudioVolumeMute,
            WinitKeyCode::AudioVolumeUp => KeyCode::AudioVolumeUp,
            WinitKeyCode::WakeUp => KeyCode::WakeUp,
            WinitKeyCode::Meta => KeyCode::Super,
            WinitKeyCode::Hyper => KeyCode::Hyper,
            WinitKeyCode::Turbo => KeyCode::Turbo,
            WinitKeyCode::Abort => KeyCode::Abort,
            WinitKeyCode::Resume => KeyCode::Resume,
            WinitKeyCode::Suspend => KeyCode::Suspend,
            WinitKeyCode::Again => KeyCode::Again,
            WinitKeyCode::Copy => KeyCode::Copy,
            WinitKeyCode::Cut => KeyCode::Cut,
            WinitKeyCode::Find => KeyCode::Find,
            WinitKeyCode::Open => KeyCode::Open,
            WinitKeyCode::Paste => KeyCode::Paste,
            WinitKeyCode::Props => KeyCode::Props,
            WinitKeyCode::Select => KeyCode::Select,
            WinitKeyCode::Undo => KeyCode::Undo,
            WinitKeyCode::Hiragana => KeyCode::Hiragana,
            WinitKeyCode::Katakana => KeyCode::Katakana,
            WinitKeyCode::F1 => KeyCode::F1,
            WinitKeyCode::F2 => KeyCode::F2,
            WinitKeyCode::F3 => KeyCode::F3,
            WinitKeyCode::F4 => KeyCode::F4,
            WinitKeyCode::F5 => KeyCode::F5,
            WinitKeyCode::F6 => KeyCode::F6,
            WinitKeyCode::F7 => KeyCode::F7,
            WinitKeyCode::F8 => KeyCode::F8,
            WinitKeyCode::F9 => KeyCode::F9,
            WinitKeyCode::F10 => KeyCode::F10,
            WinitKeyCode::F11 => KeyCode::F11,
            WinitKeyCode::F12 => KeyCode::F12,
            WinitKeyCode::F13 => KeyCode::F13,
            WinitKeyCode::F14 => KeyCode::F14,
            WinitKeyCode::F15 => KeyCode::F15,
            WinitKeyCode::F16 => KeyCode::F16,
            WinitKeyCode::F17 => KeyCode::F17,
            WinitKeyCode::F18 => KeyCode::F18,
            WinitKeyCode::F19 => KeyCode::F19,
            WinitKeyCode::F20 => KeyCode::F20,
            WinitKeyCode::F21 => KeyCode::F21,
            WinitKeyCode::F22 => KeyCode::F22,
            WinitKeyCode::F23 => KeyCode::F23,
            WinitKeyCode::F24 => KeyCode::F24,
            WinitKeyCode::F25 => KeyCode::F25,
            WinitKeyCode::F26 => KeyCode::F26,
            WinitKeyCode::F27 => KeyCode::F27,
            WinitKeyCode::F28 => KeyCode::F28,
            WinitKeyCode::F29 => KeyCode::F29,
            WinitKeyCode::F30 => KeyCode::F30,
            WinitKeyCode::F31 => KeyCode::F31,
            WinitKeyCode::F32 => KeyCode::F32,
            WinitKeyCode::F33 => KeyCode::F33,
            WinitKeyCode::F34 => KeyCode::F34,
            WinitKeyCode::F35 => KeyCode::F35,
            _ => return None,
        }),
        WinitPhysicalKey::Unidentified(_) => None,
    }
}
