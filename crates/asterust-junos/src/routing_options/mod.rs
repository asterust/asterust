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
//! - [aggregate (Routing)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/aggregate-edit-routing-options.html)
//! - [generate](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/generate-edit-routing-options.html)
//! - [martians](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/martians-edit-routing-options.html)
//! - [rib (General)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/rib-edit-routing-options.html)
//! - [instance-import](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/instance-import-edit-routing-options.html)
//! - [instance-export](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/instance-export-edit-routing-options.html)
//! - [flex-algorithm](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/flex-algorithm-edit-routing-options.html)
//! - [definition](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/definition-edit-routing-options-flex-algorithm.html)
//! - [flex-algorithm (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/flex-algorithm-ospf-edit-routing-options.html)
//! - [definition (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/definition-ospf-edit-routing-options.html)
//! - [Configuring Flexible Algorithm for Segment Routing](https://networkingdocs.hpe.com/documentation/us/en/software/junos/is-is/segment-routing/topics/task/isis-configuring-flex-algo-sr.html)
//! - [Flexible Algorithms in IS-IS for Segment Routing Traffic Engineering](https://networkingdocs.hpe.com/documentation/us/en/software/junos/is-is/topics/topic-map/infocus-isis-flex-algo-sr.html)
//! - [How to Configure Flexible Algorithms in OSPF for Segment Routing Traffic Engineering](https://networkingdocs.hpe.com/documentation/us/en/software/junos/ospf/topics/topic-map/ospf-flex-algo-sr.html)
//! - [srv6 (Routing Options)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/srv6-edit-routing-options-source-packet-routing.html)
//! - [locator](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/locator-edit-routing-options-source-packet-routing-srv6.html)
//! - [micro-sid](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/routing-options-source-packet-routing-srv6-locator-micro-sid.html)
//! - [block](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/routing-options-source-packet-routing-srv6-block.html)

mod aggregate;
mod auto_export;
mod autonomous_system;
mod confederation;
mod fate_sharing;
mod flex_algorithm;
mod forwarding_table;
mod generate;
mod graceful_restart;
mod interface_routes;
mod martians;
mod maximum_paths;
mod resolution;
mod rib;
mod rib_groups;
mod route_attributes;
mod srv6;
mod static_routes;

pub use aggregate::{AggregateRoute, AggregateRoutes};
pub use auto_export::{AutoExport, AutoExportFamily, AutoExportScope};
pub use autonomous_system::AutonomousSystemOptions;
pub use confederation::Confederation;
pub use fate_sharing::{FateSharingEntry, FateSharingGroup};
pub use flex_algorithm::{FlexAlgorithm, FlexAlgorithmDefinition};
pub use forwarding_table::ForwardingTable;
pub use generate::{GenerateRoute, GenerateRoutes};
pub use graceful_restart::GracefulRestart;
pub use interface_routes::{InterfaceRoutes, InterfaceRoutesFamily};
pub use martians::Martian;
pub use maximum_paths::MaximumPaths;
pub use resolution::{Resolution, ResolutionRib};
pub use rib::Rib;
pub use rib_groups::RibGroup;
pub use route_attributes::RouteAttributes;
pub use srv6::{Srv6, Srv6Block, Srv6Locator, Srv6MicroSid};
pub use static_routes::{QualifiedNextHop, StaticRoute, StaticRoutes};

use crate::{arg_of, args_of, child_block, first_word, has_leaf, list_values, statement_words, top_level};
use aggregate::parse_aggregate_routes;
use asterust::config::Config;
use auto_export::parse_auto_export;
use autonomous_system::parse_autonomous_system_options;
use confederation::parse_confederation;
use fate_sharing::parse_fate_sharing;
use flex_algorithm::parse_flex_algorithms;
use forwarding_table::parse_forwarding_table;
use generate::parse_generate_routes;
use graceful_restart::parse_graceful_restart;
use interface_routes::parse_interface_routes;
use martians::parse_martians;
use maximum_paths::parse_maximum_paths;
use resolution::parse_resolution;
use rib::parse_ribs;
use rib_groups::parse_rib_groups;
use srv6::parse_srv6;
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
    pub aggregate: Option<AggregateRoutes>,
    pub generate: Option<GenerateRoutes>,
    pub martians: Vec<Martian>,
    pub ribs: Vec<Rib>,
    pub rib_groups: Vec<RibGroup>,
    pub instance_import: Vec<String>,
    pub instance_export: Vec<String>,
    pub auto_export: Option<AutoExport>,
    pub fate_sharing: Vec<FateSharingGroup>,
    pub flex_algorithms: Vec<FlexAlgorithm>,
    /// `source-packet-routing srv6`.
    pub srv6: Option<Srv6>,
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
        aggregate: child_block(root, "aggregate").map(parse_aggregate_routes),
        generate: child_block(root, "generate").map(parse_generate_routes),
        martians: parse_martians(root),
        ribs: parse_ribs(root),
        rib_groups: parse_rib_groups(root),
        instance_import: args_of(root, "instance-import")
            .into_iter()
            .flat_map(list_values)
            .collect(),
        instance_export: args_of(root, "instance-export")
            .into_iter()
            .flat_map(list_values)
            .collect(),
        auto_export: child_block(root, "auto-export").map(parse_auto_export),
        fate_sharing: child_block(root, "fate-sharing")
            .map(parse_fate_sharing)
            .unwrap_or_default(),
        flex_algorithms: parse_flex_algorithms(root),
        srv6: parse_srv6(root),
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

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_instance_import_and_export_lists() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    instance-import [ IMPORT-A IMPORT-B ];\n    instance-export EXPORT-A;\n}\n",
        ))
        .unwrap();
        assert_eq!(options.instance_import, ["IMPORT-A", "IMPORT-B"]);
        assert_eq!(options.instance_export, ["EXPORT-A"]);
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
