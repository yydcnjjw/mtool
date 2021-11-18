use serde::{Deserialize, Serialize};

fn default_algorithm() -> String {
    return "TC3-HMAC-SHA256".into();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credential {
    pub secret_id: String,
    pub secret_key: String,
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
}

impl Credential {
    pub fn new(secret_id: String, secret_key: String) -> Self {
        Self {
            secret_id,
            secret_key,
            algorithm: "TC3-HMAC-SHA256".into(),
        }
    }
}
