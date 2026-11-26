//! `netconf`
//! `system { services { netconf { ... } } }`.
//!
//! # See also
//!
//! - [netconf](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/netconf-edit-system-services.html)
//! - [ssh (NETCONF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/ssh-edit-system-services-netconf.html)

use crate::{arg_of, child_block, has_leaf};
use asterust::config::ConfigLine;

/// The `netconf ssh` block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct NetconfSsh {
    pub port: Option<String>,
    pub connection_limit: Option<String>,
    pub rate_limit: Option<String>,
}

/// The `system services netconf` settings.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Netconf {
    pub ssh: Option<NetconfSsh>,
    pub rfc_compliant: bool,
}

pub(super) fn parse_netconf(netconf: &ConfigLine) -> Netconf {
    Netconf {
        ssh: child_block(netconf, "ssh").map(|ssh| NetconfSsh {
            port: arg_of(ssh, "port").map(str::to_string),
            connection_limit: arg_of(ssh, "connection-limit").map(str::to_string),
            rate_limit: arg_of(ssh, "rate-limit").map(str::to_string),
        }),
        rfc_compliant: has_leaf(netconf, "rfc-compliant"),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    #[test]
    fn bare_netconf_ssh_has_no_settings() {
        let netconf = system(&juniper::parse_braces(
            "system {\n    services {\n        netconf {\n            ssh;\n        }\n    }\n}\n",
        ))
        .unwrap()
        .services
        .unwrap()
        .netconf
        .unwrap();
        let ssh = netconf.ssh.unwrap();
        assert_eq!(ssh.port, None);
        assert!(!netconf.rfc_compliant);
    }
}
