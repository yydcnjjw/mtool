pub mod egress;
pub mod ingress;
pub mod routing;
pub mod tls;
pub mod transport;
pub mod protos;

use self::{egress::EgressConfig, ingress::IngressConfig, routing::RoutingConfig};
use mapp::serde::{Deserialize, Serialize};

#[cfg(not(target_family = "wasm"))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct AppConfig {
    pub ingress: Vec<IngressConfig>,
    pub egress: Vec<EgressConfig>,
    pub routing: RoutingConfig,
}
