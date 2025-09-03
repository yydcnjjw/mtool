use std::net::IpAddr;

use mapp::serde::Deserialize;

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(crate = "mapp::serde")]
pub(crate) struct Config {
    pub peer_id: Option<String>,
    pub listen_port: Option<u16>,
    #[serde(default)]
    pub listen_addr_list: Vec<IpAddr>,
    #[serde(default)]
    pub listen_iface_name_list: Vec<String>,
    #[serde(default)]
    pub listen_iface_index_list: Vec<u32>,

    pub boot_node: Option<BootNode>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "mapp::serde")]
pub(crate) struct BootNode {
    pub peer_id: String,
    pub address: String,
}
