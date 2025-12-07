use mapp::{
    anyhow::{self, anyhow},
    itertools::Itertools,
    keyboard_types::{Key, Modifiers, NamedKey},
    nom::{
        self,
        branch::alt,
        bytes::complete::take_while1,
        character::complete::{char, one_of, satisfy},
        error::ParseError,
        multi::{fold_many0, separated_list1},
        sequence::{delimited, pair, terminated},
        AsChar, Input, Parser,
    },
};
use std::{
    fmt,
    hash::Hash,
    ops::{Deref, DerefMut},
    str::FromStr,
};

#[derive(Debug, Clone)]
pub struct KeySequence(Vec<CombineKey>);

impl KeySequence {
    pub fn empty() -> Self {
        Self(Vec::new())
    }
}

impl Deref for KeySequence {
    type Target = Vec<CombineKey>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KeySequence {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for KeySequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.iter().map(|key| key.to_string()).join(" "))
    }
}

impl From<&[CombineKey]> for KeySequence {
    fn from(value: &[CombineKey]) -> Self {
        Self(value.iter().cloned().collect())
    }
}

impl<'a> TryFrom<&'a str> for KeySequence {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        key_sequence::<&str, nom::error::Error<&str>>(value)
            .map(|(_, kseq)| Self(kseq))
            .map_err(|e| anyhow!("{e:?}"))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CombineKey {
    pub key: Key,
    pub modifiers: Modifiers,
}

impl fmt::Display for CombineKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let modifiers = self.modifiers;

        if modifiers.contains(Modifiers::ALT) {
            write!(f, "A-")?;
        }

        if modifiers.contains(Modifiers::CONTROL) {
            write!(f, "C-")?;
        }

        if modifiers.contains(Modifiers::HYPER) {
            write!(f, "H-")?;
        }

        if modifiers.contains(Modifiers::META) {
            write!(f, "M-")?;
        }

        if modifiers.contains(Modifiers::SHIFT) {
            write!(f, "S-")?;
        }

        if modifiers.contains(Modifiers::SUPER) {
            write!(f, "s-")?;
        }

        match &self.key {
            Key::Character(ch) => write!(f, "{}", ch),
            Key::Named(named_key) => write!(f, "<{}>", named_key),
        }
    }
}

fn named_key<I, E>(input: I) -> nom::IResult<I, Key, E>
where
    I: Input + AsRef<str>,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
{
    delimited(
        char('<'),
        take_while1(|c: <I as Input>::Item| !matches!(c.as_char(), '<' | '>' | ' ' | '\n' | '\t')),
        char('>'),
    )
    .map_opt(|s: I| {
        NamedKey::from_str(s.as_ref())
            .map(|key| Key::Named(key))
            .ok()
    })
    .parse(input)
}

fn char_key<I, E>(input: I) -> nom::IResult<I, Key, E>
where
    I: nom::Input,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
{
    satisfy(|c| c.is_ascii() && !c.is_ascii_control())
        .map(|c| Key::Character(c.to_string()))
        .parse(input)
}

fn key<I, E>(input: I) -> nom::IResult<I, Key, E>
where
    I: nom::Input + AsRef<str>,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
{
    alt((named_key, char_key)).parse(input)
}

fn modifier<I, E>(input: I) -> nom::IResult<I, Modifiers, E>
where
    I: nom::Input,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
{
    one_of("ACHMsS")
        .map_opt(|ch| {
            Some(match ch {
                'A' => Modifiers::ALT,
                'C' => Modifiers::CONTROL,
                'H' => Modifiers::HYPER,
                'M' => Modifiers::META,
                'S' => Modifiers::SHIFT,
                's' => Modifiers::SUPER,
                _ => return None,
            })
        })
        .parse(input)
}

fn combine_key<I, E>(input: I) -> nom::IResult<I, CombineKey, E>
where
    I: nom::Input + AsRef<str>,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
{
    pair(
        fold_many0(
            terminated(modifier, char('-')),
            Modifiers::empty,
            |acc, item| acc | item,
        ),
        key,
    )
    .map(|(modifiers, key)| CombineKey { key, modifiers })
    .parse(input)
}

fn key_sequence<I, E>(input: I) -> nom::IResult<I, Vec<CombineKey>, E>
where
    I: nom::Input + AsRef<str>,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
{
    separated_list1(char(' '), combine_key).parse(input)
}
