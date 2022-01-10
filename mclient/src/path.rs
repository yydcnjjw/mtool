use std::path::PathBuf;

pub fn config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|p| p.join(".mtool"))
}

pub fn config_file() -> Option<PathBuf> {
    config_dir().map(|p| p.join("config.toml"))
}

pub fn logger_config_file() -> Option<PathBuf> {
    config_dir().map(|p| p.join("log4rs.yaml"))
}
