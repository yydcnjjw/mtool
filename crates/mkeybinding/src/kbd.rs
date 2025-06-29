use std::{
    fmt,
    hash::Hash,
    ops::{ControlFlow, Deref, DerefMut},
    str::FromStr,
};

use mapp::{
    anyhow::{self, anyhow, Context},
    keyboard_types::{Code, Modifiers},
    nom::{
        self,
        branch::alt,
        bytes::complete::{tag, tag_no_case, take_while},
        character::{complete::anychar, is_alphanumeric},
        combinator::{map, map_res},
        multi::separated_list1,
        sequence::delimited,
    },
};

trait ParseKbd: Sized {
    fn parse_kbd(s: &str) -> Result<Self, anyhow::Error>;
}

impl ParseKbd for Code {
    fn parse_kbd(s: &str) -> Result<Self, anyhow::Error> {
        let r: nom::IResult<&str, Code> = alt((
            map_res(
                delimited(
                    tag_no_case("<f"),
                    nom::character::streaming::digit1,
                    tag(">"),
                ),
                |d| -> Result<Code, anyhow::Error> {
                    let n = u8::from_str(d).context("Parse fn")?;
                    Ok(match n {
                        1 => Code::F1,
                        2 => Code::F2,
                        3 => Code::F3,
                        4 => Code::F4,
                        5 => Code::F5,
                        6 => Code::F6,
                        7 => Code::F7,
                        8 => Code::F8,
                        9 => Code::F9,
                        10 => Code::F10,
                        11 => Code::F11,
                        12 => Code::F12,
                        _ => Err(anyhow!("fn < 12: {}", n))?,
                    })
                },
            ),
            map(tag_no_case("<Backspace>"), |_| Code::Backspace),
            map(tag_no_case("<Return>"), |_| Code::Enter),
            map(tag_no_case("<Spacebar>"), |_| Code::Space),
            map(tag_no_case("<Escape>"), |_| Code::Escape),
            // TODO: more special keycode
            map_res(anychar, |c| -> Result<Code, anyhow::Error> {
                Ok(match c {
                    '`' => Code::Backquote,
                    '1' => Code::Digit1,
                    '2' => Code::Digit2,
                    '3' => Code::Digit3,
                    '4' => Code::Digit4,
                    '5' => Code::Digit5,
                    '6' => Code::Digit6,
                    '7' => Code::Digit7,
                    '8' => Code::Digit8,
                    '9' => Code::Digit9,
                    '0' => Code::Digit0,
                    '-' => Code::Minus,
                    '=' => Code::Equal,
                    'q' => Code::KeyQ,
                    'w' => Code::KeyW,
                    'e' => Code::KeyE,
                    'r' => Code::KeyR,
                    't' => Code::KeyT,
                    'y' => Code::KeyY,
                    'u' => Code::KeyU,
                    'i' => Code::KeyI,
                    'o' => Code::KeyO,
                    'p' => Code::KeyP,
                    '[' => Code::BracketLeft,
                    ']' => Code::BracketRight,
                    '\\' => Code::Backslash,
                    'a' => Code::KeyA,
                    's' => Code::KeyS,
                    'd' => Code::KeyD,
                    'f' => Code::KeyF,
                    'g' => Code::KeyG,
                    'h' => Code::KeyH,
                    'j' => Code::KeyJ,
                    'k' => Code::KeyK,
                    'l' => Code::KeyL,
                    ';' => Code::Semicolon,
                    '\'' => Code::Quote,
                    'z' => Code::KeyZ,
                    'x' => Code::KeyX,
                    'c' => Code::KeyC,
                    'v' => Code::KeyV,
                    'b' => Code::KeyB,
                    'n' => Code::KeyN,
                    'm' => Code::KeyM,
                    ',' => Code::Comma,
                    '.' => Code::Period,
                    '/' => Code::Slash,
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

impl ParseKbd for Modifiers {
    fn parse_kbd(s: &str) -> Result<Self, anyhow::Error> {
        match s {
            "S" => Ok(Modifiers::SHIFT),
            "C" => Ok(Modifiers::CONTROL),
            "M" => Ok(Modifiers::SUPER),
            "A" => Ok(Modifiers::ALT),
            _ => Err(anyhow!("Unknown ModifierState: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub struct KeyCombine {
    pub code: Code,
    pub mods: Modifiers,
}

const IGNORE_MODS: Modifiers =
    Modifiers::from_bits_truncate(Modifiers::NUM_LOCK.bits() | Modifiers::CAPS_LOCK.bits());

impl Hash for KeyCombine {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.code.hash(state);
        (self.mods | IGNORE_MODS).hash(state);
    }
}

impl PartialEq for KeyCombine {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code && self.mods | IGNORE_MODS == other.mods | IGNORE_MODS
    }
}

impl fmt::Display for KeyCombine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mods = self.mods;

        if mods.contains(Modifiers::SHIFT) {
            write!(f, "S-")?;
        }

        if mods.contains(Modifiers::CONTROL) {
            write!(f, "C-")?;
        }

        if mods.contains(Modifiers::SUPER) {
            write!(f, "M-")?;
        }

        if mods.contains(Modifiers::ALT) {
            write!(f, "A-")?;
        }

        if mods.contains(Modifiers::CAPS_LOCK) {
            write!(f, "CapsLock-")?;
        }

        if mods.contains(Modifiers::NUM_LOCK) {
            write!(f, "NumLock-")?;
        }

        let code = self.code.to_string();
        if code.starts_with("Key") {
            write!(f, "{}", code.trim_start_matches("Key").to_lowercase())
        } else {
            write!(f, "<{}>", code)
        }
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
            let key = Code::parse_kbd(last)?;

            let kms = rest
                .iter()
                .try_fold(Modifiers::empty(), |kms: Modifiers, s| {
                    let km = Modifiers::parse_kbd(s);

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
                    kcseq.push(KeyCombine {
                        code: key,
                        mods: kms,
                    });
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
