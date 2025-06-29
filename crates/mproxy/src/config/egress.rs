use mapp::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct EgressConfig {
    pub id: String,
    #[serde(flatten)]
    pub client: ClientConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ClientConfig {
    Http(http::ClientConfig),
    Direct(direct::ClientConfig),
}

pub mod http {
    use mapp::serde::{Deserialize, Serialize};

    use crate::config::transport::ConnectorConfig;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(crate = "mapp::serde")]
    pub struct ClientConfig {
        #[serde(flatten)]
        pub connector: ConnectorConfig,
    }
}

pub mod direct {
    use mapp::serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(crate = "mapp::serde")]
    pub struct ClientConfig {}
}
