mapp::cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        pub mod egress;
        pub mod ingress;
        pub mod routing;
        pub mod transport;
        pub mod tls;

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
    }
}

pub mod protos;
