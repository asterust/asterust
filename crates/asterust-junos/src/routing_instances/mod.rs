//! `routing-instances`
//! `routing-instances { ... }`.
//!
//! # See also
//!
//! - [`[edit routing-instances]` hierarchy](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/topic-map/hierarchy-edit-routing-instances.html)
//! - [instance-type](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/instance-type-edit-routing-instances-vp.html)
//! - [route-distinguisher](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/route-distinguisher-edit-routing-instances-vp.html)
//! - [vrf-target](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/vrf-target-edit-routing-instances-vp.html)
//! - [vrf-import](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/vrf-import-edit-routing-instances-vp.html)
//! - [vrf-export](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/vrf-export-edit-routing-instances-vp.html)

use crate::protocols::bgp::{parse_srv6, BgpSrv6};
use crate::{active_children, arg_of, args_of, child_block, list_values, top_level};
use asterust::config::{Config, ConfigLine};

/// A routing instance. Of the protocol configuration nested inside the instance only `bgp_srv6` is extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct RoutingInstance {
    pub name: String,
    /// e.g. `"vrf"`, `"virtual-router"`.
    pub instance_type: Option<String>,
    pub description: Option<String>,
    pub route_distinguisher: Option<String>,
    pub interfaces: Vec<String>,
    /// From `vrf-target <target>` (both directions) or `vrf-target { import ...; }`.
    pub vrf_target_import: Option<String>,
    pub vrf_target_export: Option<String>,
    /// Policies from `vrf-import` / `vrf-export`.
    pub vrf_import_policies: Vec<String>,
    pub vrf_export_policies: Vec<String>,
    /// `protocols bgp source-packet-routing srv6`, the SRv6 service SIDs of the VRF.
    pub bgp_srv6: Option<BgpSrv6>,
}

impl RoutingInstance {
    /// Whether this is a VRF (`instance-type vrf`).
    pub fn is_vrf(&self) -> bool {
        self.instance_type.as_deref() == Some("vrf")
    }
}

/// Returns every instance under the top-level `routing-instances` block.
///
/// # Examples
///
/// ```
/// use asterust::config::juniper;
/// use asterust_junos::routing_instances::routing_instances;
///
/// let config = juniper::parse_braces("routing-instances {\n    CUSTOMER-A {\n        instance-type vrf;\n    }\n}\n");
/// let instances = routing_instances(&config);
/// assert_eq!(instances[0].name, "CUSTOMER-A");
/// assert!(instances[0].is_vrf());
/// ```
pub fn routing_instances(config: &Config) -> Vec<RoutingInstance> {
    let Some(root) = top_level(config, "routing-instances") else {
        return Vec::new();
    };
    // Instance names have no spaces; this skips `apply-groups` and similar statements.
    active_children(root)
        .filter(|c| !c.text.contains(' '))
        .map(parse_instance)
        .collect()
}

fn parse_instance(line: &ConfigLine) -> RoutingInstance {
    let target_both = arg_of(line, "vrf-target");
    let target_block = child_block(line, "vrf-target");
    let target = |direction: &str| -> Option<String> {
        target_both
            .or_else(|| target_block.and_then(|b| arg_of(b, direction)))
            .map(str::to_string)
    };
    let policies =
        |keyword: &str| -> Vec<String> { args_of(line, keyword).into_iter().flat_map(list_values).collect() };

    RoutingInstance {
        name: line.text.clone(),
        instance_type: arg_of(line, "instance-type").map(str::to_string),
        description: arg_of(line, "description").map(str::to_string),
        route_distinguisher: arg_of(line, "route-distinguisher").map(str::to_string),
        interfaces: args_of(line, "interface").into_iter().map(str::to_string).collect(),
        vrf_target_import: target("import"),
        vrf_target_export: target("export"),
        vrf_import_policies: policies("vrf-import"),
        vrf_export_policies: policies("vrf-export"),
        bgp_srv6: child_block(line, "protocols")
            .and_then(|protocols| child_block(protocols, "bgp"))
            .and_then(parse_srv6),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::juniper;

    // Written from JunOS hierarchy knowledge, not adapted from documentation; see the README "Sources".
    const CONFIG: &str = r#"
routing-instances {
    CUSTOMER-A {
        instance-type vrf;
        interface ge-0/0/1.100;
        route-distinguisher 65000:1;
        vrf-target target:65000:1;
    }
    CUSTOMER-B {
        description "Customer B";
        instance-type vrf;
        interface ge-0/0/2.0;
        interface ge-0/0/3.0;
        route-distinguisher 65000:2;
        vrf-import CUSTB-IN;
        vrf-export [ CUSTB-OUT TAG-ROUTES ];
    }
    CUSTOMER-C {
        instance-type vrf;
        route-distinguisher 65000:3;
        vrf-target {
            import target:65000:3;
            export target:65000:33;
        }
    }
    MGMT {
        instance-type virtual-router;
    }
}
"#;

    #[test]
    fn vrf_target_in_both_directions() {
        let instances = routing_instances(&juniper::parse_braces(CONFIG));
        assert_eq!(instances.len(), 4);

        let a = &instances[0];
        assert_eq!(a.name, "CUSTOMER-A");
        assert!(a.is_vrf());
        assert_eq!(a.route_distinguisher.as_deref(), Some("65000:1"));
        assert_eq!(a.interfaces, ["ge-0/0/1.100"]);
        assert_eq!(a.vrf_target_import.as_deref(), Some("target:65000:1"));
        assert_eq!(a.vrf_target_export.as_deref(), Some("target:65000:1"));
    }

    #[test]
    fn policies_target_block_and_non_vrf_instance() {
        let instances = routing_instances(&juniper::parse_braces(CONFIG));

        let b = &instances[1];
        assert_eq!(b.description.as_deref(), Some("Customer B"));
        assert_eq!(b.interfaces, ["ge-0/0/2.0", "ge-0/0/3.0"]);
        assert_eq!(b.vrf_import_policies, ["CUSTB-IN"]);
        assert_eq!(b.vrf_export_policies, ["CUSTB-OUT", "TAG-ROUTES"]);
        assert_eq!(b.vrf_target_import, None);

        let c = &instances[2];
        assert_eq!(c.vrf_target_import.as_deref(), Some("target:65000:3"));
        assert_eq!(c.vrf_target_export.as_deref(), Some("target:65000:33"));

        assert!(!instances[3].is_vrf());
    }
}
