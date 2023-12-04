use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EgressConfig {
    pub id: String,
    #[serde(flatten)]
    pub client: ClientConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ClientConfig {
    Http(http::ClientConfig),
    Direct(direct::ClientConfig),
}

pub mod http {
    use serde::{Deserialize, Serialize};

    use crate::config::transport::ConnectorConfig;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ClientConfig {
        #[serde(flatten)]
        pub connector: ConnectorConfig,
    }
}

pub mod direct {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ClientConfig {}
}
