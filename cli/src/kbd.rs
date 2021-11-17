use std::{ops::ControlFlow, str::FromStr};

use bitflags::bitflags;
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while},
    character::{complete::anychar, is_alphanumeric},
    combinator::map,
    multi::separated_list1,
    sequence::delimited,
};

use thiserror::Error;

// keyboard definition.
#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Parse(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Hash, PartialEq, std::cmp::Eq, Clone)]
pub enum Key {
    Fn(u8),
    Char(char),
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    Null,
    Esc,
}

impl FromStr for Key {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let r: nom::IResult<&str, Key> = alt((
            map(
                delimited(
                    tag_no_case("<f"),
                    nom::character::streaming::digit1,
                    tag(">"),
                ),
                |d| Key::Fn(u8::from_str(d).unwrap()),
            ),
            map(tag_no_case("<Backspace>"), |_| Key::Backspace),
            map(tag_no_case("<Enter>"), |_| Key::Enter),
            map(tag_no_case("<Left>"), |_| Key::Left),
            map(tag_no_case("<Right>"), |_| Key::Right),
            map(tag_no_case("<Up>"), |_| Key::Up),
            map(tag_no_case("<Down>"), |_| Key::Down),
            map(tag_no_case("<Home>"), |_| Key::Home),
            map(tag_no_case("<End>"), |_| Key::End),
            map(tag_no_case("<PageUp>"), |_| Key::PageUp),
            map(tag_no_case("<PageDown>"), |_| Key::PageDown),
            map(tag_no_case("<Tab>"), |_| Key::Tab),
            map(tag_no_case("<BackTab>"), |_| Key::BackTab),
            map(tag_no_case("<Delete>"), |_| Key::Delete),
            map(tag_no_case("<Insert>"), |_| Key::Insert),
            map(tag_no_case("<Null>"), |_| Key::Null),
            map(tag_no_case("<Esc>"), |_| Key::Esc),
            map(anychar, |c| Key::Char(c)),
        ))(s);

        if let Err(e) = r {
            return Err(Error::Parse(format!("Key FromStr: {}", e)));
        }

        let (_, key) = r.unwrap();

        Ok(key)
    }
}

bitflags! {
 pub struct KeyMods : u32 {
     const L_SHIFT = 0x00000001;
     const L_META = 0x00000002;
     const L_CTRL = 0x00000004;
     const L_ALT = 0x00000008;

     const R_SHIFT = 0x00010000;
     const R_META = 0x00020000;
     const R_CTRL = 0x00040000;
     const R_ALT = 0x00080000;

     const SHIFT = Self::L_SHIFT.bits | Self::R_SHIFT.bits;
     const META = Self::L_META.bits | Self::R_META.bits;
     const CTRL = Self::L_CTRL.bits | Self::R_CTRL.bits;
     const ALT = Self::L_ALT.bits | Self::R_ALT.bits;
     const NONE = 0x0;
 }
}

impl FromStr for KeyMods {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "S" => Ok(KeyMods::SHIFT),
            "C" => Ok(KeyMods::CTRL),
            "M" => Ok(KeyMods::META),
            "A" => Ok(KeyMods::ALT),
            _ => Err(Error::Parse(format!("Unknown keyMod: {}", s))),
        }
    }
}

#[derive(Debug)]
pub struct KeyCombine {
    pub key: Key,
    pub mods: KeyMods,
}

pub fn parse_kbd(in_: &str) -> Result<Vec<KeyCombine>> {
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

    let mut kcs = Vec::new();
    for kc in o {
        assert!(kc.len() >= 1);

        let (last, rest) = kc.split_last().unwrap();
        let key = Key::from_str(last)?;

        let kms = rest.iter().try_fold(KeyMods::NONE, |kms: KeyMods, s| {
            let km = KeyMods::from_str(s);

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
                kcs.push(KeyCombine { key, mods: kms });
            }
            ControlFlow::Break(e) => return Err(e),
        }
    }

    Ok(kcs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdb() {
        let kcs = parse_kbd("C-M-a C-S-<enter> C-<f1> b").unwrap();
        println!("{:?}", kcs);
    }
}
