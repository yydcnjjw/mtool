use mapp::serde::Deserialize;

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(crate = "mapp::serde")]
pub(crate) struct Config {
    pub peer_id: Option<String>,
    pub listen_port: Option<u16>,

    pub boot_node: Option<BootNode>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "mapp::serde")]
pub(crate) struct BootNode {
    pub peer_id: String,
    pub address: String,
}
