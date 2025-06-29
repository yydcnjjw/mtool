use std::path::PathBuf;

use mapp::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct RoutingConfig {
    pub resource: Vec<PathBuf>,
    pub rule: Vec<RuleConfig>,
    pub default_rule: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct RuleConfig {
    pub id: String,
    pub target: Vec<String>,
    pub src: Vec<String>,
    pub dest: String,
}
