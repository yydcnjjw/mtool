use mapp::serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "mapp::serde")]
pub(crate) struct Config {
    pub(crate) listen: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen: "0.0.0.0:50051".into(),
        }
    }
}
