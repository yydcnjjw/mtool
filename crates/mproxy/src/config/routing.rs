use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub resource: Vec<PathBuf>,
    pub rule: Vec<RuleConfig>
}


#[derive(Debug, Serialize, Deserialize)]
pub struct RuleConfig {
    pub id: String,
    pub target: Vec<String>,
    pub src: Vec<String>,
    pub dest: String,
}
