use crate::keydef::{KeyCode, KeyModifier};
use std::os::raw::{c_uint, c_ulong};
use x11::{keysym, xlib};
use xproto::KeyButMask;

pub struct KeySym(pub c_uint);

impl KeySym {
    pub fn new(v: c_ulong) -> Self {
        Self(c_uint::try_from(v).unwrap())
    }
}

impl From<KeySym> for KeyCode {
    fn from(v: KeySym) -> Self {
        match v.0 {
            keysym::XK_grave => KeyCode::GraveAccent,
            keysym::XK_1 => KeyCode::Num1,
            keysym::XK_2 => KeyCode::Num2,
            keysym::XK_3 => KeyCode::Num3,
            keysym::XK_4 => KeyCode::Num4,
            keysym::XK_5 => KeyCode::Num5,
            keysym::XK_6 => KeyCode::Num6,
            keysym::XK_7 => KeyCode::Num7,
            keysym::XK_8 => KeyCode::Num8,
            keysym::XK_9 => KeyCode::Num9,
            keysym::XK_0 => KeyCode::Num0,
            keysym::XK_minus => KeyCode::Minus,
            keysym::XK_equal => KeyCode::Equal,
            keysym::XK_BackSpace => KeyCode::BackSpace,
            keysym::XK_Tab => KeyCode::Tab,
            keysym::XK_q => KeyCode::Q,
            keysym::XK_w => KeyCode::W,
            keysym::XK_e => KeyCode::E,
            keysym::XK_r => KeyCode::R,
            keysym::XK_t => KeyCode::T,
            keysym::XK_y => KeyCode::Y,
            keysym::XK_u => KeyCode::U,
            keysym::XK_i => KeyCode::I,
            keysym::XK_o => KeyCode::O,
            keysym::XK_p => KeyCode::P,
            keysym::XK_bracketleft => KeyCode::BracketLeft,
            keysym::XK_bracketright => KeyCode::BracketRight,
            keysym::XK_backslash => KeyCode::Backslash,
            keysym::XK_Caps_Lock => KeyCode::CapsLock,
            keysym::XK_a => KeyCode::A,
            keysym::XK_s => KeyCode::S,
            keysym::XK_d => KeyCode::D,
            keysym::XK_f => KeyCode::F,
            keysym::XK_g => KeyCode::G,
            keysym::XK_h => KeyCode::H,
            keysym::XK_j => KeyCode::J,
            keysym::XK_k => KeyCode::K,
            keysym::XK_l => KeyCode::L,
            keysym::XK_semicolon => KeyCode::Semicolon,
            keysym::XK_apostrophe => KeyCode::Apostrophe,
            // NonUS,
            keysym::XK_Return => KeyCode::Return,
            keysym::XK_Shift_L => KeyCode::LeftShift,
            // NonUS2,
            keysym::XK_z => KeyCode::Z,
            keysym::XK_x => KeyCode::X,
            keysym::XK_c => KeyCode::C,
            keysym::XK_v => KeyCode::V,
            keysym::XK_b => KeyCode::B,
            keysym::XK_n => KeyCode::N,
            keysym::XK_m => KeyCode::M,
            keysym::XK_comma => KeyCode::Comma,
            keysym::XK_period => KeyCode::Period,
            keysym::XK_slash => KeyCode::Slash,
            // Internation1,
            keysym::XK_Shift_R => KeyCode::RightShift,
            keysym::XK_Control_L => KeyCode::LeftControl,
            keysym::XK_Alt_L => KeyCode::LeftAlt,
            keysym::XK_space => KeyCode::Spacebar,
            keysym::XK_Alt_R => KeyCode::RightAlt,
            keysym::XK_Control_R => KeyCode::RightControl,
            keysym::XK_Insert => KeyCode::Insert,
            keysym::XK_Delete => KeyCode::Delete,
            keysym::XK_leftarrow => KeyCode::LeftArrow,
            keysym::XK_Home => KeyCode::Home,
            keysym::XK_End => KeyCode::End,
            keysym::XK_uparrow => KeyCode::UpArrow,
            keysym::XK_downarrow => KeyCode::DownArrow,
            keysym::XK_Page_Up => KeyCode::PageUp,
            keysym::XK_Page_Down => KeyCode::PageDown,
            keysym::XK_rightarrow => KeyCode::RightArrow,
            keysym::XK_Num_Lock => KeyCode::NumLock,
            keysym::XK_KP_7 => KeyCode::Keypad7,
            keysym::XK_KP_4 => KeyCode::Keypad4,
            keysym::XK_KP_1 => KeyCode::Keypad1,
            keysym::XK_KP_Divide => KeyCode::Divide,
            keysym::XK_KP_8 => KeyCode::Keypad8,
            keysym::XK_KP_5 => KeyCode::Keypad5,
            keysym::XK_KP_2 => KeyCode::Keypad2,
            keysym::XK_KP_0 => KeyCode::Keypad0,
            keysym::XK_KP_Multiply => KeyCode::Multiply,
            keysym::XK_KP_9 => KeyCode::Keypad9,
            keysym::XK_KP_6 => KeyCode::Keypad6,
            keysym::XK_KP_3 => KeyCode::Keypad3,
            // _ => KeypadPeriod,
            keysym::XK_KP_Subtract => KeyCode::Subtract,
            keysym::XK_KP_Add => KeyCode::Add,
            // _ => KeypadComma,
            keysym::XK_KP_Enter => KeyCode::KeypadEnter,
            keysym::XK_Escape => KeyCode::Escape,
            keysym::XK_F1 => KeyCode::F1,
            keysym::XK_F2 => KeyCode::F2,
            keysym::XK_F3 => KeyCode::F3,
            keysym::XK_F4 => KeyCode::F4,
            keysym::XK_F5 => KeyCode::F5,
            keysym::XK_F6 => KeyCode::F6,
            keysym::XK_F7 => KeyCode::F7,
            keysym::XK_F8 => KeyCode::F8,
            keysym::XK_F9 => KeyCode::F9,
            keysym::XK_F10 => KeyCode::F10,
            keysym::XK_F11 => KeyCode::F11,
            keysym::XK_F12 => KeyCode::F12,
            keysym::XK_Print => KeyCode::PrintScreen,
            keysym::XK_Scroll_Lock => KeyCode::ScrollLock,
            keysym::XK_Pause => KeyCode::Pause,
            keysym::XK_Super_L => KeyCode::LeftGUI,
            keysym::XK_Super_R => KeyCode::RightGUI,
            keysym::XK_Menu => KeyCode::Application,
            _ => KeyCode::Unknown,
        }
    }
}

// shift       Shift_L (0x32),  Shift_R (0x3e)
// lock        Caps_Lock (0x42)
// control     Control_L (0x25),  Control_R (0x69)
// mod1        Alt_L (0x40),  Alt_R (0x6c),  Meta_L (0xcd)
// mod2        Num_Lock (0x4d)
// mod3
// mod4        Super_L (0x85),  Super_R (0x86),  Super_L (0xce),  Hyper_L (0xcf)
// mod5        ISO_Level3_Shift (0x5c),  Mode_switch (0xcb)
impl From<KeyButMask> for KeyModifier {
    fn from(v: KeyButMask) -> Self {
        let v = v as u32;
        let mut mods = KeyModifier::NONE;
        if v & xlib::ShiftMask != 0 {
            mods |= KeyModifier::SHIFT;
        }
        if v & xlib::LockMask != 0 {
            mods |= KeyModifier::CAPSLOCK;
        }
        if v & xlib::ControlMask != 0 {
            mods |= KeyModifier::CONTROL;
        }
        if v & xlib::Mod1Mask != 0 {
            mods |= KeyModifier::ALT;
        }
        if v & xlib::Mod2Mask != 0 {
            mods |= KeyModifier::NUMLOCK;
        }
        if v & xlib::Mod4Mask != 0 {
            mods |= KeyModifier::SUPER;
        }
        return mods;
    }
}
