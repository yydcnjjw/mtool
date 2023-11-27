cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        pub mod egress;
        pub mod ingress;
        pub mod routing;
        pub mod transport;

        use self::{egress::EgressConfig, ingress::IngressConfig, routing::RoutingConfig};
        use serde::{Deserialize, Serialize};

        #[cfg(not(target_family = "wasm"))]
        #[derive(Debug, Serialize, Deserialize)]
        pub struct AppConfig {
            pub ingress: Vec<IngressConfig>,
            pub egress: Vec<EgressConfig>,
            pub routing: RoutingConfig,
        }
    }
}

pub mod protos;
