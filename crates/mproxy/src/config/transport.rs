use std::{fs::File, io::BufReader, path::PathBuf, sync::Arc};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "transport")]
#[serde(rename_all = "lowercase")]
pub enum AcceptorConfig {
    Quic(quic::AcceptorConfig),
    Tcp(tcp::AcceptorConfig),
    Kcp(kcp::AcceptorConfig),
    Tls(Box<tls::AcceptorConfig>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "transport")]
#[serde(rename_all = "lowercase")]
pub enum ConnectorConfig {
    Quic(quic::ConnectorConfig),
    Tcp(tcp::ConnectorConfig),
    Kcp(kcp::ConnectorConfig),
    Tls(Box<tls::ConnectorConfig>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Endpoint {
    Single { address: String, port: u16 },
    Multi { address: String, port_range: String },
}

pub mod quic {
    use std::net::SocketAddr;

    use serde::{Deserialize, Serialize};

    use super::{Endpoint, TlsConfig};

    #[derive(Debug, Serialize, Deserialize, Clone)]
    #[serde(rename_all = "lowercase")]
    #[serde(tag = "type")]
    pub enum CongestionType {
        Bbr { initial_window: Option<u64> },
        Cubic,
        NewReno,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct TransportConfig {
        pub keep_alive_interval: Option<u64>,
        pub congestion: Option<CongestionType>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct StatsConfig {
        pub interval: usize,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct AcceptorConfig {
        pub listen: SocketAddr,
        pub tls: TlsConfig,

        #[serde(flatten)]
        pub transport: TransportConfig,

        pub stats: Option<StatsConfig>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
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

    use serde::{Deserialize, Serialize};

    use super::Endpoint;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct AcceptorConfig {
        pub listen: SocketAddr,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ConnectorConfig {
        pub endpoint: Endpoint,
    }
}

pub mod kcp {
    use std::{net::SocketAddr, time::Duration};

    use serde::{Deserialize, Serialize};
    use serde_with::serde_as;
    use tokio_kcp::{KcpConfig, KcpNoDelayConfig};

    use super::Endpoint;

    #[derive(Serialize, Deserialize)]
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
    #[serde(remote = "KcpConfig")]
    #[serde_as]
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
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct AcceptorConfig {
        pub listen: SocketAddr,
        #[serde(with = "KcpConfigDef", default)]
        pub kcp: KcpConfig,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ConnectorConfig {
        pub endpoint: Endpoint,
        #[serde(with = "KcpConfigDef", default)]
        pub kcp: KcpConfig,
    }
}

pub mod tls {
    use serde::{Deserialize, Serialize};

    use super::TlsConfig;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct AcceptorConfig {
        pub next_layer: super::AcceptorConfig,
        pub tls: TlsConfig,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ConnectorConfig {
        pub next_layer: super::ConnectorConfig,
        pub tls: TlsConfig,
        pub server_name: String,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TlsConfig {
    pub ca_cert: PathBuf,
    pub key: Option<PathBuf>,
    pub cert: Option<PathBuf>,
}

#[cfg(not(target_family = "wasm"))]
impl TlsConfig {
    pub fn root_cert_store(&self) -> Result<rustls::RootCertStore, anyhow::Error> {
        let f = &self.ca_cert;
        let mut roots = rustls::RootCertStore::empty();

        let mut f = BufReader::new(File::open(f).context(format!("open {}", f.to_string_lossy()))?);

        for cert in rustls_pemfile::certs(&mut f)?
            .into_iter()
            .map(rustls::Certificate)
        {
            roots.add(&cert)?;
        }
        Ok(roots)
    }

    pub fn cert_chain(&self) -> Result<Vec<rustls::Certificate>, anyhow::Error> {
        let f = self
            .cert
            .as_ref()
            .context("server cert is not configured")?;
        let mut f = BufReader::new(File::open(f).context(format!("open {}", f.to_string_lossy()))?);
        Ok(rustls_pemfile::certs(&mut f)?
            .into_iter()
            .map(rustls::Certificate)
            .collect())
    }

    pub fn key(&self) -> Result<rustls::PrivateKey, anyhow::Error> {
        let f = self
            .key
            .as_ref()
            .context("server private key is not configured")?;
        let mut f = BufReader::new(File::open(f).context(format!("open {}", f.to_string_lossy()))?);
        let mut keys = rustls_pemfile::pkcs8_private_keys(&mut f)?;
        assert_eq!(keys.len(), 1);
        Ok(rustls::PrivateKey(keys.remove(0)))
    }
}

#[cfg(not(target_family = "wasm"))]
impl TryFrom<&TlsConfig> for rustls::ServerConfig {
    type Error = anyhow::Error;

    fn try_from(c: &TlsConfig) -> Result<Self, Self::Error> {
        use rustls::server::AllowAnyAuthenticatedClient;
        let (roots, certs, key) = (c.root_cert_store()?, c.cert_chain()?, c.key()?);

        Ok(rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(Arc::new(AllowAnyAuthenticatedClient::new(roots)))
            .with_single_cert(certs, key)?)
    }
}

#[cfg(not(target_family = "wasm"))]
impl TryFrom<&TlsConfig> for rustls::ClientConfig {
    type Error = anyhow::Error;

    fn try_from(c: &TlsConfig) -> Result<Self, Self::Error> {
        let (roots, certs, key) = (c.root_cert_store()?, c.cert_chain()?, c.key()?);

        rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(roots)
            .with_client_auth_cert(certs, key)
            .context("Failed to build tls client config")
    }
}
