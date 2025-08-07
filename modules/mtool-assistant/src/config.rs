use mapp::serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(crate = "mapp::serde")]
pub(crate) struct Config {
    pub mobile_rpc_address: Option<String>,
    pub desktop_rpc_address: Option<String>,
}
