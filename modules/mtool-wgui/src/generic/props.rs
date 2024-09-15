use std::{fmt::Display, ops::Deref};

use mapp::serde::{Deserialize, Deserializer, Serialize};
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq, Serialize)]
#[serde(crate = "mapp::serde")]
pub struct Props<T>
where
    T: PartialEq + Serialize,
{
    pub data: T,
}

impl<'a, T> Deserialize<'a> for Props<T>
where
    T: PartialEq + Serialize + Deserialize<'a>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        Ok(Props::new(T::deserialize(deserializer)?))
    }
}

impl<T> Props<T>
where
    T: PartialEq + Serialize,
{
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl<T> Deref for Props<T>
where
    T: PartialEq + Serialize,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> From<T> for Props<T>
where
    T: PartialEq + Serialize,
{
    fn from(value: T) -> Self {
        Props::new(value)
    }
}

impl<T: Display> Display for Props<T>
where
    T: PartialEq + Serialize,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}
