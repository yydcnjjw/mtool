use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub source: Vec<PathBuf>,
    pub rule: Vec<RuleConfig>
}


#[derive(Debug, Serialize, Deserialize)]
pub struct RuleConfig {
    pub target: Vec<String>,
    pub source: Vec<String>,
    pub dest: String,
}
