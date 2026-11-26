//! `routing-options { ... }`.

use crate::{arg_of, first_word, top_level};
use asterust::config::Config;

/// Global routing options.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct RoutingOptions {
    pub router_id: Option<String>,
    pub autonomous_system: Option<String>,
}

/// The top-level `routing-options` block, or `None` if it is absent.
pub fn routing_options(config: &Config) -> Option<RoutingOptions> {
    let root = top_level(config, "routing-options")?;
    Some(RoutingOptions {
        router_id: arg_of(root, "router-id").map(str::to_string),
        autonomous_system: arg_of(root, "autonomous-system")
            .and_then(first_word)
            .map(str::to_string),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::juniper;

    #[test]
    fn extracts_router_id_and_asn() {
        // Written from JunOS hierarchy knowledge, not adapted from documentation; see the README "Sources".
        let config = juniper::parse_braces(
            "routing-options {\n    router-id 10.255.0.1;\n    autonomous-system 65000 loops 2;\n}\n",
        );
        let options = routing_options(&config).unwrap();
        assert_eq!(options.router_id.as_deref(), Some("10.255.0.1"));
        assert_eq!(options.autonomous_system.as_deref(), Some("65000"));
    }

    #[test]
    fn missing_block_is_none() {
        assert!(routing_options(&juniper::parse_braces("system {\n host-name r1;\n}\n")).is_none());
    }

    #[test]
    fn doc_router_id_and_autonomous_system() {
        // Adapted from Juniper's BGP User Guide, "BGP Peering Sessions", "Example: Configuring Internal BGP Peer
        // Sessions", Device A `show routing-options`; see the README "Sources".
        let config =
            juniper::parse_braces("routing-options {\n    router-id 192.168.6.5;\n    autonomous-system 17;\n}\n");
        let options = routing_options(&config).unwrap();
        assert_eq!(options.router_id.as_deref(), Some("192.168.6.5"));
        assert_eq!(options.autonomous_system.as_deref(), Some("17"));
    }
}
