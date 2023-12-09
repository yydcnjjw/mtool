use anyhow::Context;
use itertools::Itertools;
use quinn::rustls as quic_tls;
use rustls::server::WebPkiClientVerifier;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, path::PathBuf, sync::Arc};

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

        for cert in rustls_pemfile::certs(&mut f) {
            roots.add(cert?)?;
        }
        Ok(roots)
    }

    pub fn cert_chain(&self) -> Result<Vec<CertificateDer<'static>>, anyhow::Error> {
        let f = self
            .cert
            .as_ref()
            .context("server cert is not configured")?;
        let mut f = BufReader::new(File::open(f).context(format!("open {}", f.to_string_lossy()))?);
        Ok(rustls_pemfile::certs(&mut f).try_collect()?)
    }

    pub fn key(&self) -> Result<PrivateKeyDer<'static>, anyhow::Error> {
        let f = self
            .key
            .as_ref()
            .context("server private key is not configured")?;
        let mut f = BufReader::new(File::open(f).context(format!("open {}", f.to_string_lossy()))?);
        rustls_pemfile::private_key(&mut f)?.context("private key is none")
    }

    pub fn quic_root_cert_store(&self) -> Result<quic_tls::RootCertStore, anyhow::Error> {
        let f = &self.ca_cert;
        let mut roots = quic_tls::RootCertStore::empty();

        let mut f = BufReader::new(File::open(f).context(format!("open {}", f.to_string_lossy()))?);

        for cert in rustls_pemfile::certs(&mut f) {
            roots.add(&quic_tls::Certificate(cert?.to_vec()))?;
        }
        Ok(roots)
    }

    pub fn quic_cert_chain(&self) -> Result<Vec<quic_tls::Certificate>, anyhow::Error> {
        let f = self
            .cert
            .as_ref()
            .context("server cert is not configured")?;
        let mut f = BufReader::new(File::open(f).context(format!("open {}", f.to_string_lossy()))?);
        Ok(rustls_pemfile::certs(&mut f)
            .map_ok(|cert| quic_tls::Certificate(cert.to_vec()))
            .try_collect()?)
    }

    pub fn quic_key(&self) -> Result<quic_tls::PrivateKey, anyhow::Error> {
        let f = self
            .key
            .as_ref()
            .context("server private key is not configured")?;
        let mut f = BufReader::new(File::open(f).context(format!("open {}", f.to_string_lossy()))?);
        let key = rustls_pemfile::private_key(&mut f)?.context("private key is none")?;
        Ok(quic_tls::PrivateKey(key.secret_der().to_vec()))
    }
}

impl TryFrom<&TlsConfig> for quic_tls::ServerConfig {
    type Error = anyhow::Error;

    fn try_from(c: &TlsConfig) -> Result<Self, Self::Error> {
        use quic_tls::server::AllowAnyAuthenticatedClient;
        let (roots, certs, key) = (
            c.quic_root_cert_store()?,
            c.quic_cert_chain()?,
            c.quic_key()?,
        );

        Ok(quic_tls::ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(Arc::new(AllowAnyAuthenticatedClient::new(roots)))
            .with_single_cert(certs, key)?)
    }
}

impl TryFrom<&TlsConfig> for quic_tls::ClientConfig {
    type Error = anyhow::Error;

    fn try_from(c: &TlsConfig) -> Result<Self, Self::Error> {
        let (roots, certs, key) = (
            c.quic_root_cert_store()?,
            c.quic_cert_chain()?,
            c.quic_key()?,
        );

        quic_tls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(roots)
            .with_client_auth_cert(certs, key)
            .context("Failed to build tls client config")
    }
}

impl TryFrom<&TlsConfig> for rustls::ServerConfig {
    type Error = anyhow::Error;

    fn try_from(c: &TlsConfig) -> Result<Self, Self::Error> {
        let (roots, certs, key) = (c.root_cert_store()?, c.cert_chain()?, c.key()?);

        Ok(rustls::ServerConfig::builder()
            .with_client_cert_verifier(WebPkiClientVerifier::builder(Arc::new(roots)).build()?)
            .with_single_cert(certs, key)?)
    }
}

impl TryFrom<&TlsConfig> for rustls::ClientConfig {
    type Error = anyhow::Error;

    fn try_from(c: &TlsConfig) -> Result<Self, Self::Error> {
        let (roots, certs, key) = (c.root_cert_store()?, c.cert_chain()?, c.key()?);

        rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_client_auth_cert(certs, key)
            .context("Failed to build tls client config")
    }
}
