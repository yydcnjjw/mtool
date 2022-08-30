use core::fmt;
use std::{
    hash::Hash,
    ops::{ControlFlow, Deref, DerefMut},
    str::FromStr,
};

use anyhow::Context;
use msysev::keydef::{KeyCode, KeyModifier};

use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while},
    character::{complete::anychar, is_alphanumeric},
    combinator::{map, map_res},
    multi::separated_list1,
    sequence::delimited,
};

use thiserror::Error;

use lazy_static::lazy_static;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Parse(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

trait TryFromStr: Sized {
    fn from_str(s: &str) -> Result<Self>;
}

impl TryFromStr for KeyCode {
    fn from_str(s: &str) -> Result<Self> {
        let r: nom::IResult<&str, KeyCode> = alt((
            map_res(
                delimited(
                    tag_no_case("<f"),
                    nom::character::streaming::digit1,
                    tag(">"),
                ),
                |d| -> Result<KeyCode> {
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
                        _ => Err(Error::Parse(format!("fn < 12: {}", n)))?,
                    })
                },
            ),
            map(tag_no_case("<Backspace>"), |_| KeyCode::BackSpace),
            map(tag_no_case("<Return>"), |_| KeyCode::Return),
            map(tag_no_case("<Spacebar>"), |_| KeyCode::Spacebar),
            // TODO: more special keycode
            map_res(anychar, |c| -> Result<KeyCode> {
                Ok(match c {
                    '`' => KeyCode::GraveAccent,
                    '1' => KeyCode::Num1,
                    '2' => KeyCode::Num2,
                    '3' => KeyCode::Num3,
                    '4' => KeyCode::Num4,
                    '5' => KeyCode::Num5,
                    '6' => KeyCode::Num6,
                    '7' => KeyCode::Num7,
                    '8' => KeyCode::Num8,
                    '9' => KeyCode::Num9,
                    '0' => KeyCode::Num0,
                    '-' => KeyCode::Minus,
                    '=' => KeyCode::Equal,
                    'q' => KeyCode::Q,
                    'w' => KeyCode::W,
                    'e' => KeyCode::E,
                    'r' => KeyCode::R,
                    't' => KeyCode::T,
                    'y' => KeyCode::Y,
                    'u' => KeyCode::U,
                    'i' => KeyCode::I,
                    'o' => KeyCode::O,
                    'p' => KeyCode::P,
                    '[' => KeyCode::BracketLeft,
                    ']' => KeyCode::BracketRight,
                    '\\' => KeyCode::Backslash,
                    'a' => KeyCode::A,
                    's' => KeyCode::S,
                    'd' => KeyCode::D,
                    'f' => KeyCode::F,
                    'g' => KeyCode::G,
                    'h' => KeyCode::H,
                    'j' => KeyCode::J,
                    'k' => KeyCode::K,
                    'l' => KeyCode::L,
                    ';' => KeyCode::Semicolon,
                    '\'' => KeyCode::Apostrophe,
                    'z' => KeyCode::Z,
                    'x' => KeyCode::X,
                    'c' => KeyCode::C,
                    'v' => KeyCode::V,
                    'b' => KeyCode::B,
                    'n' => KeyCode::N,
                    'm' => KeyCode::M,
                    ',' => KeyCode::Comma,
                    '.' => KeyCode::Period,
                    '/' => KeyCode::Slash,
                    _ => Err(Error::Parse(format!("Unknown char: {}", c)))?,
                })
            }),
        ))(s);

        if let Err(e) = r {
            return Err(Error::Parse(format!("Key FromStr: {}", e)));
        }

        let (_, key) = r.unwrap();

        Ok(key)
    }
}

impl TryFromStr for KeyModifier {
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "S" => Ok(KeyModifier::SHIFT),
            "C" => Ok(KeyModifier::CONTROL),
            "M" => Ok(KeyModifier::SUPER),
            "A" => Ok(KeyModifier::ALT),
            _ => Err(Error::Parse(format!("Unknown keyMod: {}", s))),
        }
    }
}

#[derive(Debug, Clone, std::cmp::Eq)]
pub struct KeyCombine {
    pub key: KeyCode,
    pub mods: KeyModifier,
}

lazy_static! {
    static ref IGNORE_MODS: KeyModifier = KeyModifier::NUMLOCK | KeyModifier::CAPSLOCK;
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

        if mods.contains(KeyModifier::SHIFT) {
            write!(f, "S-")?;
        }

        if mods.contains(KeyModifier::CONTROL) {
            write!(f, "C-")?;
        }

        if mods.contains(KeyModifier::SUPER) {
            write!(f, "M-")?;
        }

        if mods.contains(KeyModifier::ALT) {
            write!(f, "A-")?;
        }

        if mods.contains(KeyModifier::CAPSLOCK) {
            write!(f, "CapsLock-")?;
        }

        if mods.contains(KeyModifier::NUMLOCK) {
            write!(f, "NumLock-")?;
        }

        write!(f, "{}", format!("{:?}", self.key).to_lowercase())
    }
}

#[derive(Debug, Clone)]
pub struct KeySequence {
    inner: Vec<KeyCombine>,
}

impl KeySequence {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn parse(in_: &str) -> Result<Self> {
        let r: nom::IResult<&str, Vec<Vec<&str>>> = separated_list1(
            tag(" "),
            separated_list1(
                tag("-"),
                take_while(|c: char| is_alphanumeric(c as u8) || c == '<' || c == '>'),
            ),
        )(in_);

        if let Err(e) = r {
            return Err(Error::Parse(format!("parse kbd: {}", e)));
        }

        let (in_, o) = r.unwrap();

        if in_.len() != 0 {
            return Err(Error::Parse(format!("Unknown rest content: {}", in_)));
        }

        let mut kcseq = Self::new();
        for kc in o {
            assert!(kc.len() >= 1);

            let (last, rest) = kc.split_last().unwrap();
            let key = KeyCode::from_str(last)?;

            let kms = rest
                .iter()
                .try_fold(KeyModifier::NONE, |kms: KeyModifier, s| {
                    let km = KeyModifier::from_str(s);

                    if let Err(e) = km {
                        return ControlFlow::Break(e);
                    }

                    let km = km.unwrap();

                    if kms.contains(km) {
                        ControlFlow::Break(Error::Parse(format!("{} Repeat definition", s)))
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
    fn to_key_sequence(self) -> Result<KeySequence>;
}

impl ToKeySequence for &KeySequence {
    fn to_key_sequence(self) -> Result<KeySequence> {
        Ok(self.clone())
    }
}

impl ToKeySequence for &[KeyCombine] {
    fn to_key_sequence(self) -> Result<KeySequence> {
        Ok(KeySequence {
            inner: self.to_vec(),
        })
    }
}

impl ToKeySequence for KeyCombine {
    fn to_key_sequence(self) -> Result<KeySequence> {
        Ok(KeySequence { inner: vec![self] })
    }
}

impl ToKeySequence for Vec<KeyCombine> {
    fn to_key_sequence(self) -> Result<KeySequence> {
        Ok(KeySequence { inner: self })
    }
}

impl ToKeySequence for &str {
    fn to_key_sequence(self) -> Result<KeySequence> {
        KeySequence::parse(self)
    }
}

impl ToKeySequence for String {
    fn to_key_sequence(self) -> Result<KeySequence> {
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
