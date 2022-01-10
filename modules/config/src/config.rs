use std::{
    fmt::{self, Display},
    path::PathBuf,
    sync::Arc,
};

use tokio::fs;

use anyhow::Context;

use toml::Value;

use crate::{Error, SerdeResult, Service};

pub struct Config {
    table: toml::value::Table,
}

impl Config {
    // BUG: https://github.com/rust-lang/rust/issues/50133
    pub async fn new<T>(path: T) -> Arc<Self>
    where
        T: Into<PathBuf>,
    {
        let path: PathBuf = path.into();

        let s = fs::read_to_string(path.as_path()).await.unwrap_or_default();
        let table = toml::from_str(&s).unwrap_or_default();

        Arc::new(Self { table })
    }
}

impl Service for Config {
    fn get_value(self: Arc<Self>, key: String) -> SerdeResult<Value> {
        self.table
            .get(&key)
            .cloned()
            .ok_or(serde_error::Error::new(&Error::KeyNotFound(
                key.to_string(),
            )))
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(s) = toml::to_string_pretty(&self.table).context("Serialize config") {
            write!(f, "{}", s)
        } else {
            Err(fmt::Error)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use tempfile::NamedTempFile;

    static DEFAULT_MOCK_CONFIG: &str = r#"
[module1]
value = "*"

[module2]
value = "*"
"#;

    fn prepare_config_file(s: &str) -> anyhow::Result<NamedTempFile> {
        let mut file = NamedTempFile::new()?;
        write!(file, "{}", s)?;
        Ok(file)
    }

    async fn new_config(s: &str) -> Arc<Config> {
        let file = prepare_config_file(s).unwrap();
        Config::new(file.path()).await
    }

    #[tokio::test]
    async fn test_config_get() {
        let config = new_config(DEFAULT_MOCK_CONFIG).await;
        let module1 = config.clone().get("module1".into()).unwrap();
        assert_eq!(module1["value"].as_str(), Some("*"));
        let module2 = config.clone().get("module2".into()).unwrap();
        assert_eq!(module2["value"].as_str(), Some("*"));
    }
}
