//! Extracts the `dhcp-local-server` settings of `system services`.
//!
//! Configuration path: `system { services { dhcp-local-server { ... } } }`.
//!
//! # See also
//!
//! - [dhcp-local-server](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/dhcp-local-server-edit-system-services.html)

use crate::{args_of, child_block, named_children};
use asterust::config::ConfigLine;

/// A `group <name>` of `dhcp-local-server`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DhcpLocalServerGroup {
    pub name: String,
    /// Each `interface` as written.
    pub interfaces: Vec<String>,
}

/// The `system services dhcp-local-server` groups. The many per-group options are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DhcpLocalServer {
    pub groups: Vec<DhcpLocalServerGroup>,
    /// Groups under `dhcpv6`.
    pub dhcpv6_groups: Vec<DhcpLocalServerGroup>,
}

pub(super) fn parse_dhcp_local_server(server: &ConfigLine) -> DhcpLocalServer {
    let groups = |parent: &ConfigLine| -> Vec<DhcpLocalServerGroup> {
        named_children(parent, "group")
            .map(|(name, group)| DhcpLocalServerGroup {
                name: name.to_string(),
                interfaces: args_of(group, "interface").into_iter().map(str::to_string).collect(),
            })
            .collect()
    };
    DhcpLocalServer {
        groups: groups(server),
        dhcpv6_groups: child_block(server, "dhcpv6").map(groups).unwrap_or_default(),
    }
}
