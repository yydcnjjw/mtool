use cloud_api::tencent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Credential {
    pub secret_id: String,
    pub secret_key: String,
}

impl From<Credential> for tencent::credential::Credential {
    fn from(cred: Credential) -> Self {
        Self::new(cred.secret_id, cred.secret_key)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub credential: Credential,
}
