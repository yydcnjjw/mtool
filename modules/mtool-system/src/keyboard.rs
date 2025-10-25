use mapp::keyboard_types::{Code, KeyState, Modifiers};

pub(crate) fn update_modifier_state(code: &Code, state: &KeyState) -> Modifiers {
    static mut MODIFIERS: Modifiers = Modifiers::empty();

    let modifer = match code {
        Code::ShiftLeft | Code::ShiftRight => Modifiers::SHIFT,
        Code::CapsLock => Modifiers::CAPS_LOCK,
        Code::ControlLeft | Code::ControlRight => Modifiers::CONTROL,
        Code::AltLeft | Code::AltRight => Modifiers::ALT,
        Code::NumLock => Modifiers::NUM_LOCK,
        Code::Super => Modifiers::SUPER,
        _ => Modifiers::empty(),
    };

    unsafe {
        match state {
            KeyState::Down => MODIFIERS |= modifer,
            KeyState::Up => MODIFIERS -= modifer,
        };
        MODIFIERS
    }
}
