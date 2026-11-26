//! `routing-options`
//! `routing-options { ... }`.
//!
//! # See also
//!
//! - [`[edit routing-options]` hierarchy](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/topic-map/hierarchy-edit-routing-options.html)
//! - [router-id](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/router-id-edit-routing-options.html)
//! - [autonomous-system](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/autonomous-system-edit-routing-options.html)
//! - [ipv6-router-id](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/ipv6-router-id-edit-routing-options.html)
//! - [nonstop-routing](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/nonstop-routing-edit-routing-options.html)
//! - [nsr-phantom-holdtime](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/nsr-phantom-holdtime-edit-routing-options.html)
//! - [options (Routing Options)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/options-edit-routing-options.html)
//! - [static (Routing Options)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/static-edit-routing-options.html)

mod autonomous_system;
mod confederation;
mod forwarding_table;
mod graceful_restart;
mod interface_routes;
mod maximum_paths;
mod resolution;
mod route_attributes;
mod static_routes;

pub use autonomous_system::AutonomousSystemOptions;
pub use confederation::Confederation;
pub use forwarding_table::ForwardingTable;
pub use graceful_restart::GracefulRestart;
pub use interface_routes::{InterfaceRoutes, InterfaceRoutesFamily};
pub use maximum_paths::MaximumPaths;
pub use resolution::{Resolution, ResolutionRib};
pub use route_attributes::RouteAttributes;
pub use static_routes::{QualifiedNextHop, StaticRoute, StaticRoutes};

use crate::{arg_of, child_block, first_word, has_leaf, statement_words, top_level};
use asterust::config::Config;
use autonomous_system::parse_autonomous_system_options;
use confederation::parse_confederation;
use forwarding_table::parse_forwarding_table;
use graceful_restart::parse_graceful_restart;
use interface_routes::parse_interface_routes;
use maximum_paths::parse_maximum_paths;
use resolution::parse_resolution;
use static_routes::parse_static_routes;

/// Global routing options.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct RoutingOptions {
    pub router_id: Option<String>,
    pub ipv6_router_id: Option<String>,
    /// The AS number as written, e.g. `"65000"` or `"1.10"`.
    pub autonomous_system: Option<String>,
    /// The options of the `autonomous-system` statement, if it is present.
    pub autonomous_system_options: Option<AutonomousSystemOptions>,
    pub confederation: Option<Confederation>,
    pub graceful_restart: Option<GracefulRestart>,
    pub nonstop_routing: bool,
    /// Seconds.
    pub nsr_phantom_holdtime: Option<String>,
    pub maximum_paths: Option<MaximumPaths>,
    pub forwarding_table: Option<ForwardingTable>,
    pub interface_routes: Option<InterfaceRoutes>,
    pub resolution: Option<Resolution>,
    pub static_routes: Option<StaticRoutes>,
    /// The `syslog` setting of `options`, as written, e.g. `"level info"` or `"upto notice"`.
    pub options_syslog: Option<String>,
}

/// Returns the top-level `routing-options` block, or `None` if it is absent.
///
/// # Examples
///
/// ```
/// use asterust::config::juniper;
/// use asterust_junos::routing_options::routing_options;
///
/// let config = juniper::parse_braces("routing-options {\n    router-id 192.0.2.1;\n    autonomous-system 65000;\n}\n");
/// let options = routing_options(&config).unwrap();
/// assert_eq!(options.router_id.as_deref(), Some("192.0.2.1"));
/// assert_eq!(options.autonomous_system.as_deref(), Some("65000"));
/// ```
pub fn routing_options(config: &Config) -> Option<RoutingOptions> {
    let root = top_level(config, "routing-options")?;
    Some(RoutingOptions {
        router_id: arg_of(root, "router-id").map(str::to_string),
        ipv6_router_id: arg_of(root, "ipv6-router-id").map(str::to_string),
        autonomous_system: arg_of(root, "autonomous-system")
            .and_then(first_word)
            .map(str::to_string),
        autonomous_system_options: parse_autonomous_system_options(root),
        confederation: parse_confederation(root),
        graceful_restart: parse_graceful_restart(root),
        nonstop_routing: has_leaf(root, "nonstop-routing"),
        nsr_phantom_holdtime: arg_of(root, "nsr-phantom-holdtime").map(str::to_string),
        maximum_paths: parse_maximum_paths(root),
        forwarding_table: child_block(root, "forwarding-table").map(parse_forwarding_table),
        interface_routes: child_block(root, "interface-routes").map(parse_interface_routes),
        resolution: child_block(root, "resolution").map(parse_resolution),
        static_routes: child_block(root, "static").map(parse_static_routes),
        options_syslog: statement_words(root, "options").and_then(|words| {
            let start = words.iter().position(|word| *word == "syslog")?;
            Some(words[start + 1..].join(" "))
        }),
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

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_ipv6_router_id_nsr_and_options_syslog() {
        let config = juniper::parse_braces(
            "routing-options {\n    ipv6-router-id 2001:db8::1;\n    nonstop-routing;\n    nsr-phantom-holdtime 60;\n    options {\n        syslog upto notice;\n    }\n}\n",
        );
        let options = routing_options(&config).unwrap();
        assert_eq!(options.ipv6_router_id.as_deref(), Some("2001:db8::1"));
        assert!(options.nonstop_routing);
        assert_eq!(options.nsr_phantom_holdtime.as_deref(), Some("60"));
        assert_eq!(options.options_syslog.as_deref(), Some("upto notice"));
    }

    #[test]
    fn global_settings_default_to_off() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    router-id 192.0.2.1;\n}\n",
        ))
        .unwrap();
        assert_eq!(options.ipv6_router_id, None);
        assert!(!options.nonstop_routing);
        assert_eq!(options.nsr_phantom_holdtime, None);
        assert_eq!(options.options_syslog, None);
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
