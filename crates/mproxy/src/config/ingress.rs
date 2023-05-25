use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IngressConfig {
    pub id: String,

    #[serde(flatten)]
    pub server: ServerConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ServerConfig {
    Http(http::ServerConfig),
    Socks(socks::ServerConfig),
}

pub mod http {
    use serde::{Deserialize, Serialize};

    use crate::config::transport::AcceptorConfig;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ServerConfig {
        #[serde(flatten)]
        pub acceptor: AcceptorConfig,
    }
}

pub mod socks {
    use serde::{Deserialize, Serialize};

    use crate::config::transport::AcceptorConfig;
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "type")]
    #[serde(rename_all = "lowercase")]
    pub enum AuthType {
        Simple { user: String, password: String },
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Socks5Config {
        pub allow_udp: Option<bool>,
        pub auth: Option<AuthType>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ServerConfig {
        #[serde(flatten)]
        pub acceptor: AcceptorConfig,

        #[serde(flatten)]
        pub socks5: Socks5Config,
    }
}
