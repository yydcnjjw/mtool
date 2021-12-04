use windows::Win32::UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::KBDLLHOOKSTRUCT};

use crate::keydef::KeyCode;

impl From<KBDLLHOOKSTRUCT> for KeyCode {
    fn from(v: KBDLLHOOKSTRUCT) -> Self {
        match VIRTUAL_KEY(v.vkCode as u16) {
            VK_OEM_3 => KeyCode::GraveAccent,
            VK_1 => KeyCode::Num1,
            VK_2 => KeyCode::Num2,
            VK_3 => KeyCode::Num3,
            VK_4 => KeyCode::Num4,
            VK_5 => KeyCode::Num5,
            VK_6 => KeyCode::Num6,
            VK_7 => KeyCode::Num7,
            VK_8 => KeyCode::Num8,
            VK_9 => KeyCode::Num9,
            VK_0 => KeyCode::Num0,
            VK_OEM_MINUS => KeyCode::Minus,
            VK_OEM_PLUS => KeyCode::Equal,
            VK_BACK => KeyCode::BackSpace,
            VK_TAB => KeyCode::Tab,
            VK_Q => KeyCode::Q,
            VK_W => KeyCode::W,
            VK_E => KeyCode::E,
            VK_R => KeyCode::R,
            VK_T => KeyCode::T,
            VK_Y => KeyCode::Y,
            VK_U => KeyCode::U,
            VK_I => KeyCode::I,
            VK_O => KeyCode::O,
            VK_P => KeyCode::P,
            VK_OEM_4 => KeyCode::BracketLeft,
            VK_OEM_6 => KeyCode::BracketRight,
            VK_OEM_5 => KeyCode::Backslash,
            VK_CAPITAL => KeyCode::CapsLock,
            VK_A => KeyCode::A,
            VK_S => KeyCode::S,
            VK_D => KeyCode::D,
            VK_F => KeyCode::F,
            VK_G => KeyCode::G,
            VK_H => KeyCode::H,
            VK_J => KeyCode::J,
            VK_K => KeyCode::K,
            VK_L => KeyCode::L,
            VK_OEM_1 => KeyCode::Semicolon,
            VK_OEM_7 => KeyCode::Apostrophe,
            // NonUS,
            VK_RETURN => KeyCode::Return,
            VK_LSHIFT => KeyCode::LeftShift,
            // NonUS2,
            VK_Z => KeyCode::Z,
            VK_X => KeyCode::X,
            VK_C => KeyCode::C,
            VK_V => KeyCode::V,
            VK_B => KeyCode::B,
            VK_N => KeyCode::N,
            VK_M => KeyCode::M,
            VK_OEM_COMMA => KeyCode::Comma,
            VK_OEM_PERIOD => KeyCode::Period,
            VK_OEM_2 => KeyCode::Slash,
            // Internation1,
            VK_RSHIFT => KeyCode::RightShift,
            VK_LCONTROL => KeyCode::LeftControl,
            VK_LMENU => KeyCode::LeftAlt,
            VK_SPACE => KeyCode::Spacebar,
            VK_RMENU => KeyCode::RightAlt,
            VK_RCONTROL => KeyCode::RightControl,
            // VK_Insert => KeyCode::Insert,
            // VK_Delete => KeyCode::Delete,
            // VK_leftarrow => KeyCode::LeftArrow,
            // VK_Home => KeyCode::Home,
            // VK_End => KeyCode::End,
            // VK_uparrow => KeyCode::UpArrow,
            // VK_downarrow => KeyCode::DownArrow,
            // VK_Page_Up => KeyCode::PageUp,
            // VK_Page_Down => KeyCode::PageDown,
            // VK_rightarrow => KeyCode::RightArrow,
            // VK_Num_Lock => KeyCode::NumLock,
            // VK_KP_7 => KeyCode::Keypad7,
            // VK_KP_4 => KeyCode::Keypad4,
            // VK_KP_1 => KeyCode::Keypad1,
            // VK_KP_Divide => KeyCode::Divide,
            // VK_KP_8 => KeyCode::Keypad8,
            // VK_KP_5 => KeyCode::Keypad5,
            // VK_KP_2 => KeyCode::Keypad2,
            // VK_KP_0 => KeyCode::Keypad0,
            // VK_KP_Multiply => KeyCode::Multiply,
            // VK_KP_9 => KeyCode::Keypad9,
            // VK_KP_6 => KeyCode::Keypad6,
            // VK_KP_3 => KeyCode::Keypad3,
            // // _ => KeypadPeriod,
            // VK_KP_Subtract => KeyCode::Subtract,
            // VK_KP_Add => KeyCode::Add,
            // // _ => KeypadComma,
            // VK_KP_Enter => KeyCode::KeypadEnter,
            // VK_Escape => KeyCode::Escape,
            // VK_F1 => KeyCode::F1,
            // VK_F2 => KeyCode::F2,
            // VK_F3 => KeyCode::F3,
            // VK_F4 => KeyCode::F4,
            // VK_F5 => KeyCode::F5,
            // VK_F6 => KeyCode::F6,
            // VK_F7 => KeyCode::F7,
            // VK_F8 => KeyCode::F8,
            // VK_F9 => KeyCode::F9,
            // VK_F10 => KeyCode::F10,
            // VK_F11 => KeyCode::F11,
            // VK_F12 => KeyCode::F12,
            // VK_Print => KeyCode::PrintScreen,
            // VK_Scroll_Lock => KeyCode::ScrollLock,
            // VK_Pause => KeyCode::Pause,
            // VK_Super_L => KeyCode::LeftGUI,
            // VK_Super_R => KeyCode::RightGUI,
            // VK_Menu => KeyCode::Application,
            _ => KeyCode::Unknown,
        }
    }
}
