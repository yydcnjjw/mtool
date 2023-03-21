use std::net::IpAddr;

use neli::{
    consts::{
        nl::NlmF,
        rtnl::{Ifa, RtAddrFamily, RtScope, Rtm},
        socket::NlFamily,
    },
    nl::NlPayload,
    router::synchronous::NlRouter,
    rtnl::{IfaddrmsgBuilder, RtattrBuilder},
    types::RtBuffer,
    utils::Groups, err::NlmsghdrErr,
};

use crate::netcfg::{AsIfIndex, Routing};

pub struct RoutingTool {
    sock: NlRouter,
}

impl RoutingTool {
    pub fn new() -> Result<Self, anyhow::Error> {
        let (sock, _) = NlRouter::connect(NlFamily::Route, None, Groups::empty())?;
        Ok(Self { sock })
    }
}

impl Routing for RoutingTool {
    fn add_address<IfIndex>(
        &mut self,
        if_index: IfIndex,
        ipaddr: IpAddr,
        prefix_len: u8,
    ) -> Result<(), anyhow::Error>
    where
        IfIndex: AsIfIndex,
    {
        let mut rtattrs = RtBuffer::new();

        rtattrs.push(
            RtattrBuilder::default()
                .rta_type(Ifa::Local)
                .rta_payload(match ipaddr {
                    IpAddr::V4(addr) => addr.octets().to_vec(),
                    IpAddr::V6(addr) => addr.octets().to_vec(),
                })
                .build()?,
        );

        let ifaddrmsg = IfaddrmsgBuilder::default()
            .ifa_family(if ipaddr.is_ipv4() {
                RtAddrFamily::Inet
            } else {
                RtAddrFamily::Inet6
            })
            .ifa_prefixlen(prefix_len)
            .ifa_scope(RtScope::Universe)
            .ifa_index(if_index.as_if_index())
            .rtattrs(rtattrs)
            .build()?;

        let recv = self.sock.send::<_, _, Rtm, NlmsghdrErr<>>(
            Rtm::Newaddr,
            NlmF::REQUEST | NlmF::CREATE | NlmF::EXCL,
            NlPayload::Payload(ifaddrmsg),
        )?;
        for msg in recv {
            let mut msg = msg.unwrap();
            println!("{:?}",  msg.get_err());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_add_address() {
        let mut rt = RoutingTool::new().unwrap();
        rt.add_address(1, IpAddr::from_str("2000::1").unwrap(), 128)
            .unwrap();
    }
}
