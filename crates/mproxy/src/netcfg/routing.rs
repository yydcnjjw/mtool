use std::net::IpAddr;

use super::AsIfIndex;

pub trait Routing {
    fn add_address<IfIndex>(
        &mut self,
        if_index: IfIndex,
        ipaddr: IpAddr,
        prefix_len: u8,
    ) -> Result<(), anyhow::Error>
    where
        IfIndex: AsIfIndex;
}
