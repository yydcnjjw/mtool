use std::time::Duration;

use mapp::{
    serde::{Deserialize, Serialize},
    serde_with::serde_as,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "mapp::serde")]
#[serde(tag = "transport")]
#[serde(rename_all = "lowercase")]
pub enum AcceptorConfig {
    Quic(quic::AcceptorConfig),
    Tcp(tcp::AcceptorConfig),
    Kcp(kcp::AcceptorConfig),
    Tls(Box<tls::AcceptorConfig>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "mapp::serde")]
#[serde(tag = "transport")]
#[serde(rename_all = "lowercase")]
pub enum ConnectorConfigInner {
    Quic(quic::ConnectorConfig),
    Tcp(tcp::ConnectorConfig),
    Kcp(kcp::ConnectorConfig),
    Tls(Box<tls::ConnectorConfig>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "mapp::serde")]
#[serde_as(crate = "mapp::serde_with")]
#[serde(default)]
pub struct TransportConfig {
    #[serde_as(as = "DurationSeconds")]
    pub read_timeout: Duration,

    #[serde_as(as = "DurationSeconds")]
    pub write_timeout: Duration,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            read_timeout: Duration::from_secs(10),
            write_timeout: Duration::from_secs(10),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "mapp::serde")]
#[serde(rename_all = "lowercase")]
#[serde_as(crate = "mapp::serde_with")]
pub struct ConnectorConfig {
    #[serde(flatten)]
    pub inner: ConnectorConfigInner,

    #[serde(flatten)]
    pub transport: TransportConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "mapp::serde")]
#[serde(untagged)]
pub enum Endpoint {
    Single { address: String, port: u16 },
    Multi { address: String, port_range: String },
}

pub mod quic {
    use std::net::SocketAddr;

    use mapp::serde::{Deserialize, Serialize};

    use crate::config::tls::TlsConfig;

    use super::Endpoint;

    #[derive(Debug, Serialize, Deserialize, Clone)]
    #[serde(crate = "mapp::serde")]
    #[serde(rename_all = "lowercase")]
    #[serde(tag = "type")]
    pub enum CongestionType {
        Bbr { initial_window: Option<u64> },
        Cubic,
        NewReno,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(crate = "mapp::serde")]
    pub struct TransportConfig {
        pub keep_alive_interval: Option<u64>,
        pub congestion: Option<CongestionType>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(crate = "mapp::serde")]
    pub struct StatsConfig {
        pub interval: usize,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(crate = "mapp::serde")]
    pub struct AcceptorConfig {
        pub listen: SocketAddr,
        pub tls: TlsConfig,

        #[serde(flatten)]
        pub transport: TransportConfig,

        pub stats: Option<StatsConfig>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(crate = "mapp::serde")]
    pub struct ConnectorConfig {
        pub endpoint: Endpoint,
        pub local: SocketAddr,
        pub server_name: String,
        pub tls: TlsConfig,

        #[serde(flatten)]
        pub transport: TransportConfig,

        pub stats: Option<StatsConfig>,
    }
}

pub mod tcp {
    use std::net::SocketAddr;

    use mapp::serde::{Deserialize, Serialize};

    use super::Endpoint;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(crate = "mapp::serde")]
    pub struct AcceptorConfig {
        pub listen: SocketAddr,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(crate = "mapp::serde")]
    pub struct ConnectorConfig {
        pub endpoint: Endpoint,
    }
}

pub mod kcp {
    use std::{net::SocketAddr, time::Duration};

    use mapp::{
        serde::{Deserialize, Serialize},
        serde_with::serde_as,
    };
    use tokio_kcp::{KcpConfig, KcpNoDelayConfig};

    use super::Endpoint;

    #[derive(Serialize, Deserialize)]
    #[serde(crate = "mapp::serde")]
    #[serde(remote = "KcpNoDelayConfig")]
    pub struct KcpNoDelayConfigDef {
        /// Enable nodelay
        pub nodelay: bool,
        /// Internal update interval (ms)
        pub interval: i32,
        /// ACK number to enable fast resend
        pub resend: i32,
        /// Disable congetion control
        pub nc: bool,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(crate = "mapp::serde")]
    #[serde(remote = "KcpConfig")]
    #[serde_as(crate = "mapp::serde_with")]
    struct KcpConfigDef {
        /// Max Transmission Unit
        pub mtu: usize,
        /// nodelay
        #[serde(with = "KcpNoDelayConfigDef")]
        pub nodelay: KcpNoDelayConfig,
        /// Send window size
        pub wnd_size: (u16, u16),
        /// Session expire duration, default is 90 seconds

        #[serde_as(as = "DurationSeconds")]
        pub session_expire: Duration,
        /// Flush KCP state immediately after write
        pub flush_write: bool,
        /// Flush ACKs immediately after input
        pub flush_acks_input: bool,
        /// Stream mode
        pub stream: bool,
        /// Allow recv 0 byte packet. KCP Segments with 0 byte data are skipped by default.
        pub allow_recv_empty_packet: bool,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(crate = "mapp::serde")]
    pub struct AcceptorConfig {
        pub listen: SocketAddr,
        #[serde(with = "KcpConfigDef", default, flatten)]
        pub kcp: KcpConfig,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(crate = "mapp::serde")]
    pub struct ConnectorConfig {
        pub endpoint: Endpoint,
        #[serde(with = "KcpConfigDef", default, flatten)]
        pub kcp: KcpConfig,
    }
}

pub mod tls {
    use std::time::Duration;

    use mapp::{
        serde::{Deserialize, Serialize},
        serde_with::serde_as,
    };

    use crate::config::tls::TlsConfig;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(crate = "mapp::serde")]
    pub struct AcceptorConfig {
        pub next_layer: super::AcceptorConfig,
        pub tls: TlsConfig,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(crate = "mapp::serde")]
    #[serde_as(crate = "mapp::serde_with")]
    pub struct ConnectorConfig {
        pub next_layer: super::ConnectorConfig,
        pub tls: TlsConfig,
        pub server_name: String,

        #[serde_as(as = "DurationSeconds")]
        pub handshake_timeout: Duration,
    }
}
