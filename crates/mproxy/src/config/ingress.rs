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
