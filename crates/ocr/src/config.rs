use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Credential {
    pub secret_id: String,
    pub secret_key: String,    
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub credential: Credential,
}
