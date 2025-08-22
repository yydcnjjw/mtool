use mapp::serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(crate = "mapp::serde")]
pub(crate) struct Config {}
