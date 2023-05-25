use core::fmt;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    str::FromStr,
};

use anyhow::Context;
use socksv5::{v4::SocksV4Host, v5::SocksV5Host};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Address {
    Ip(IpAddr),
    Hostname(String),
}

impl ToString for Address {
    fn to_string(&self) -> String {
        match self {
            Address::Ip(ip) => ip.to_string(),
            Address::Hostname(hostname) => hostname.clone(),
        }
    }
}

impl FromStr for Address {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match IpAddr::from_str(s) {
            Ok(v) => Address::Ip(v),
            Err(_) => Address::Hostname(s.to_string()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct NetLocation {
    pub address: Address,
    pub port: u16,
}

impl fmt::Display for NetLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.address.to_string(), self.port)
    }
}

impl From<SocketAddr> for NetLocation {
    fn from(value: SocketAddr) -> Self {
        Self {
            address: Address::Ip(value.ip()),
            port: value.port(),
        }
    }
}

impl FromStr for NetLocation {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match SocketAddr::from_str(s) {
            Ok(v) => v.into(),
            Err(_) => {
                let (host, port_str) = s.rsplit_once(':').context("invalid socket address")?;
                let port: u16 = port_str.parse().context("invalid port value")?;
                Self {
                    address: Address::Hostname(host.to_string()),
                    port,
                }
            }
        })
    }
}

impl TryFrom<SocksV5Host> for Address {
    type Error = anyhow::Error;

    fn try_from(value: SocksV5Host) -> Result<Self, Self::Error> {
        Ok(match value {
            SocksV5Host::Domain(domain) => Address::Hostname(String::from_utf8(domain)?),
            SocksV5Host::Ipv4(v4) => Address::Ip(IpAddr::V4(Ipv4Addr::from(v4))),
            SocksV5Host::Ipv6(v6) => Address::Ip(IpAddr::V6(Ipv6Addr::from(v6))),
        })
    }
}

impl TryFrom<SocksV4Host> for Address {
    type Error = anyhow::Error;

    fn try_from(value: SocksV4Host) -> Result<Self, Self::Error> {
        Ok(match value {
            SocksV4Host::Domain(domain) => Address::Hostname(String::from_utf8(domain)?),
            SocksV4Host::Ip(v4) => Address::Ip(IpAddr::V4(Ipv4Addr::from(v4))),
        })
    }
}
