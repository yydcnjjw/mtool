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
    use std::time::Duration;

    use serde::{Deserialize, Serialize};
    use serde_with::serde_as;

    use crate::config::transport::ConnectorConfig;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde_as]
    pub struct ClientConfig {
        #[serde(flatten)]
        pub connector: ConnectorConfig,

        #[serde_as(as = "DurationSeconds")]
        pub forward_timeout: Duration,
    }
}

pub mod direct {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ClientConfig {}
}
