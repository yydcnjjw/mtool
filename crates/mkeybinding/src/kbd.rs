use std::{
    fmt,
    hash::Hash,
    ops::{ControlFlow, Deref, DerefMut},
    str::FromStr,
};

use anyhow::{anyhow, Context};
use msysev::*;

use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while},
    character::{complete::anychar, is_alphanumeric},
    combinator::{map, map_res},
    multi::separated_list1,
    sequence::delimited,
};

use lazy_static::lazy_static;

trait ParseKbd: Sized {
    fn parse_kbd(s: &str) -> Result<Self, anyhow::Error>;
}

impl ParseKbd for KeyCode {
    fn parse_kbd(s: &str) -> Result<Self, anyhow::Error> {
        let r: nom::IResult<&str, KeyCode> = alt((
            map_res(
                delimited(
                    tag_no_case("<f"),
                    nom::character::streaming::digit1,
                    tag(">"),
                ),
                |d| -> Result<KeyCode, anyhow::Error> {
                    let n = u8::from_str(d).context("Parse fn")?;
                    Ok(match n {
                        1 => KeyCode::F1,
                        2 => KeyCode::F2,
                        3 => KeyCode::F3,
                        4 => KeyCode::F4,
                        5 => KeyCode::F5,
                        6 => KeyCode::F6,
                        7 => KeyCode::F7,
                        8 => KeyCode::F8,
                        9 => KeyCode::F9,
                        10 => KeyCode::F10,
                        11 => KeyCode::F11,
                        12 => KeyCode::F12,
                        _ => Err(anyhow!("fn < 12: {}", n))?,
                    })
                },
            ),
            map(tag_no_case("<Backspace>"), |_| KeyCode::Backspace),
            map(tag_no_case("<Return>"), |_| KeyCode::Enter),
            map(tag_no_case("<Spacebar>"), |_| KeyCode::Space),
            map(tag_no_case("<Escape>"), |_| KeyCode::Escape),
            // TODO: more special keycode
            map_res(anychar, |c| -> Result<KeyCode, anyhow::Error> {
                Ok(match c {
                    '`' => KeyCode::Backquote,
                    '1' => KeyCode::Digit1,
                    '2' => KeyCode::Digit2,
                    '3' => KeyCode::Digit3,
                    '4' => KeyCode::Digit4,
                    '5' => KeyCode::Digit5,
                    '6' => KeyCode::Digit6,
                    '7' => KeyCode::Digit7,
                    '8' => KeyCode::Digit8,
                    '9' => KeyCode::Digit9,
                    '0' => KeyCode::Digit0,
                    '-' => KeyCode::Minus,
                    '=' => KeyCode::Equal,
                    'q' => KeyCode::KeyQ,
                    'w' => KeyCode::KeyW,
                    'e' => KeyCode::KeyE,
                    'r' => KeyCode::KeyR,
                    't' => KeyCode::KeyT,
                    'y' => KeyCode::KeyY,
                    'u' => KeyCode::KeyU,
                    'i' => KeyCode::KeyI,
                    'o' => KeyCode::KeyO,
                    'p' => KeyCode::KeyP,
                    '[' => KeyCode::BracketLeft,
                    ']' => KeyCode::BracketRight,
                    '\\' => KeyCode::Backslash,
                    'a' => KeyCode::KeyA,
                    's' => KeyCode::KeyS,
                    'd' => KeyCode::KeyD,
                    'f' => KeyCode::KeyF,
                    'g' => KeyCode::KeyG,
                    'h' => KeyCode::KeyH,
                    'j' => KeyCode::KeyJ,
                    'k' => KeyCode::KeyK,
                    'l' => KeyCode::KeyL,
                    ';' => KeyCode::Semicolon,
                    '\'' => KeyCode::Quote,
                    'z' => KeyCode::KeyZ,
                    'x' => KeyCode::KeyX,
                    'c' => KeyCode::KeyC,
                    'v' => KeyCode::KeyV,
                    'b' => KeyCode::KeyB,
                    'n' => KeyCode::KeyN,
                    'm' => KeyCode::KeyM,
                    ',' => KeyCode::Comma,
                    '.' => KeyCode::Period,
                    '/' => KeyCode::Slash,
                    _ => Err(anyhow!("Unknown char: {}", c))?,
                })
            }),
        ))(s);

        if let Err(e) = r {
            return Err(anyhow!("parse_kbd: {}", e));
        }

        let (_, key) = r.unwrap();

        Ok(key)
    }
}

impl ParseKbd for ModifierState {
    fn parse_kbd(s: &str) -> Result<Self, anyhow::Error> {
        match s {
            "S" => Ok(ModifierState::SHIFT),
            "C" => Ok(ModifierState::CONTROL),
            "M" => Ok(ModifierState::SUPER),
            "A" => Ok(ModifierState::ALT),
            _ => Err(anyhow!("Unknown ModifierState: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub struct KeyCombine {
    pub key: KeyCode,
    pub mods: ModifierState,
}

lazy_static! {
    static ref IGNORE_MODS: ModifierState = ModifierState::NUMLOCK | ModifierState::CAPSLOCK;
}

impl Hash for KeyCombine {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        (self.mods | *IGNORE_MODS).hash(state);
    }
}

impl PartialEq for KeyCombine {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.mods | *IGNORE_MODS == other.mods | *IGNORE_MODS
    }
}

impl fmt::Display for KeyCombine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mods = self.mods;

        if mods.contains(ModifierState::SHIFT) {
            write!(f, "S-")?;
        }

        if mods.contains(ModifierState::CONTROL) {
            write!(f, "C-")?;
        }

        if mods.contains(ModifierState::SUPER) {
            write!(f, "M-")?;
        }

        if mods.contains(ModifierState::ALT) {
            write!(f, "A-")?;
        }

        if mods.contains(ModifierState::CAPSLOCK) {
            write!(f, "CapsLock-")?;
        }

        if mods.contains(ModifierState::NUMLOCK) {
            write!(f, "NumLock-")?;
        }

        write!(f, "{}", format!("{:?}", self.key).to_lowercase())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeySequence {
    inner: Vec<KeyCombine>,
}

impl KeySequence {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn parse(in_: &str) -> Result<Self, anyhow::Error> {
        let r: nom::IResult<&str, Vec<Vec<&str>>> = separated_list1(
            tag(" "),
            separated_list1(
                tag("-"),
                take_while(|c: char| is_alphanumeric(c as u8) || c == '<' || c == '>'),
            ),
        )(in_);

        if let Err(e) = r {
            return Err(anyhow!("parse kbd: {}", e));
        }

        let (in_, o) = r.unwrap();

        if in_.len() != 0 {
            return Err(anyhow!("Unknown rest content: {}", in_));
        }

        let mut kcseq = Self::new();
        for kc in o {
            assert!(kc.len() >= 1);

            let (last, rest) = kc.split_last().unwrap();
            let key = KeyCode::parse_kbd(last)?;

            let kms = rest
                .iter()
                .try_fold(ModifierState::NONE, |kms: ModifierState, s| {
                    let km = ModifierState::parse_kbd(s);

                    if let Err(e) = km {
                        return ControlFlow::Break(e);
                    }

                    let km = km.unwrap();

                    if kms.contains(km) {
                        ControlFlow::Break(anyhow!("{} Repeat definition", s))
                    } else {
                        ControlFlow::Continue(kms | km)
                    }
                });

            match kms {
                ControlFlow::Continue(kms) => {
                    kcseq.push(KeyCombine { key, mods: kms });
                }
                ControlFlow::Break(e) => return Err(e),
            }
        }

        Ok(kcseq)
    }
}

impl Deref for KeySequence {
    type Target = Vec<KeyCombine>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for KeySequence {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl fmt::Display for KeySequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (last, rest) = self.inner.split_last().unwrap();
        for kc in rest {
            write!(f, "{} ", kc)?;
        }
        write!(f, "{}", last)?;
        Ok(())
    }
}

pub trait ToKeySequence {
    fn to_key_sequence(self) -> Result<KeySequence, anyhow::Error>;
}

impl ToKeySequence for &KeySequence {
    fn to_key_sequence(self) -> Result<KeySequence, anyhow::Error> {
        Ok(self.clone())
    }
}

impl ToKeySequence for &[KeyCombine] {
    fn to_key_sequence(self) -> Result<KeySequence, anyhow::Error> {
        Ok(KeySequence {
            inner: self.to_vec(),
        })
    }
}

impl ToKeySequence for KeyCombine {
    fn to_key_sequence(self) -> Result<KeySequence, anyhow::Error> {
        Ok(KeySequence { inner: vec![self] })
    }
}

impl ToKeySequence for Vec<KeyCombine> {
    fn to_key_sequence(self) -> Result<KeySequence, anyhow::Error> {
        Ok(KeySequence { inner: self })
    }
}

impl ToKeySequence for &str {
    fn to_key_sequence(self) -> Result<KeySequence, anyhow::Error> {
        KeySequence::parse(self)
    }
}

impl ToKeySequence for String {
    fn to_key_sequence(self) -> Result<KeySequence, anyhow::Error> {
        KeySequence::parse(&self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdb() {
        "C-M-a C-S-<Return> C-<f1> b".to_key_sequence().unwrap();
        assert!("".to_key_sequence().is_err());
    }
}
