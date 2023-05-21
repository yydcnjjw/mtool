use std::{fs::File, io::BufReader, path::PathBuf, sync::Arc};

use anyhow::Context;
use rustls::server::AllowAnyAuthenticatedClient;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "transport")]
#[serde(rename_all = "lowercase")]
pub enum AcceptorConfig {
    Quic(quic::AcceptorConfig),
    Tcp(tcp::AcceptorConfig),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "transport")]
#[serde(rename_all = "lowercase")]
pub enum ConnectorConfig {
    Quic(quic::ConnectorConfig),
    Tcp(tcp::ConnectorConfig),
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
        Bbr {
            initial_window: Option<u64>,
        },
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TlsConfig {
    pub ca_cert: PathBuf,
    pub key: Option<PathBuf>,
    pub cert: Option<PathBuf>,
}

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

impl TryFrom<&TlsConfig> for rustls::ServerConfig {
    type Error = anyhow::Error;

    fn try_from(c: &TlsConfig) -> Result<Self, Self::Error> {
        let (roots, certs, key) = (c.root_cert_store()?, c.cert_chain()?, c.key()?);

        Ok(rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(Arc::new(AllowAnyAuthenticatedClient::new(roots)))
            .with_single_cert(certs, key)?)
    }
}

impl TryFrom<&TlsConfig> for rustls::ClientConfig {
    type Error = anyhow::Error;

    fn try_from(c: &TlsConfig) -> Result<Self, Self::Error> {
        let (roots, certs, key) = (c.root_cert_store()?, c.cert_chain()?, c.key()?);

        rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(roots)
            .with_single_cert(certs, key)
            .context("Failed to build tls client config")
    }
}
