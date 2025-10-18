use mapp::serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(crate = "mapp::serde")]
#[allow(unused)]
pub(crate) struct Config {}
