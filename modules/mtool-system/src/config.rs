use mapp::serde::Deserialize;

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(crate = "mapp::serde")]
pub(crate) struct Config {
    #[serde(default)]
    pub broadcast_event_list: Vec<String>,
}
