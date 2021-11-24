use bitflags::bitflags;

#[derive(Debug, Clone)]
pub enum KeyCode {
    GraveAccent = 1, // `
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,
    Minus, // -
    Equal, // =
    BackSpace = 15,
    Tab,
    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,
    BracketLeft,  // [
    BracketRight, // ]
    Backslash,    // \
    CapsLock,
    A,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,
    Semicolon,  // ;
    Apostrophe, // '
    // NonUS,
    Return = 43,
    LeftShift,
    // NonUS2,
    Z = 46,
    X,
    C,
    V,
    B,
    N,
    M,
    Comma,  // ,
    Period, // .
    Slash,  // /
    // Internation1,
    RightShift = 57,
    LeftControl,
    LeftAlt = 60,
    Spacebar,
    RightAlt,
    RightControl = 64,
    Insert = 75,
    Delete,
    LeftArrow = 79,
    Home,
    End,
    UpArrow,
    DownArrow,
    PageUp,
    PageDown,
    RightArrow = 89,
    NumLock,
    Keypad7,
    Keypad4,
    Keypad1,
    Divide = 95, // /
    Keypad8,
    Keypad5,
    Keypad2,
    Keypad0,
    Multiply,
    Keypad9,
    Keypad6,
    Keypad3,
    KeypadPeriod,
    Subtract,
    Add,
    KeypadComma,
    KeypadEnter,
    Escape = 110,
    F1 = 112,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    PrintScreen,
    ScrollLock,
    Pause,
    LeftGUI,     // super_L
    RightGUI,    // super_R
    Application, // menu
    Unknown,
}

bitflags! {
 pub struct KeyModifier : u32 {
     const SHIFT = 0x00000001;
     const CAPSLOCK = 0x00000002;
     const CONTROL = 0x00000004;
     const ALT = 0x00000008;
     const NUMLOCK = 0x00000010;
     const SUPER = 0x00000020;
     const NONE = 0x0;
 }
}
