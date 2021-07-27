#[derive(Debug)]
pub struct Credential {
    pub secret_id: String,
    pub secret_key: String,
    pub algorithm: &'static str,
}

impl Credential {
    pub fn new(secret_id: String, secret_key: String) -> Self {
        Self {
            secret_id,
            secret_key,
            algorithm: "TC3-HMAC-SHA256",
        }
    }
}
