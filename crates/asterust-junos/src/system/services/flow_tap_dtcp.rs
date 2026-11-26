//! `system { services { flow-tap-dtcp { ... } } }`.

use crate::{arg_of, child_block};
use asterust::config::ConfigLine;

/// The `ssh` block of `flow-tap-dtcp`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct FlowTapDtcpSsh {
    pub connection_limit: Option<String>,
    pub rate_limit: Option<String>,
}

/// The `system services flow-tap-dtcp` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct FlowTapDtcp {
    pub ssh: Option<FlowTapDtcpSsh>,
}

pub(super) fn parse_flow_tap_dtcp(dtcp: &ConfigLine) -> FlowTapDtcp {
    FlowTapDtcp {
        ssh: child_block(dtcp, "ssh").map(|ssh| FlowTapDtcpSsh {
            connection_limit: arg_of(ssh, "connection-limit").map(str::to_string),
            rate_limit: arg_of(ssh, "rate-limit").map(str::to_string),
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax page. See the README "Sources".
    #[test]
    fn extracts_flow_tap_dtcp_limits() {
        let dtcp = system(&juniper::parse_braces(
            "system {\n    services {\n        flow-tap-dtcp {\n            ssh {\n                connection-limit 50;\n                rate-limit 100;\n            }\n        }\n    }\n}\n",
        ))
        .unwrap()
        .services
        .unwrap()
        .flow_tap_dtcp
        .unwrap();
        let ssh = dtcp.ssh.unwrap();
        assert_eq!(ssh.connection_limit.as_deref(), Some("50"));
        assert_eq!(ssh.rate_limit.as_deref(), Some("100"));
    }

    #[test]
    fn bare_flow_tap_dtcp_ssh_has_no_limits() {
        let dtcp = system(&juniper::parse_braces(
            "system {\n    services {\n        flow-tap-dtcp {\n            ssh;\n        }\n    }\n}\n",
        ))
        .unwrap()
        .services
        .unwrap()
        .flow_tap_dtcp
        .unwrap();
        let ssh = dtcp.ssh.unwrap();
        assert_eq!(ssh.connection_limit, None);
        assert_eq!(ssh.rate_limit, None);
    }

    #[test]
    fn no_flow_tap_dtcp_is_none() {
        let services = system(&juniper::parse_braces(
            "system {\n    services {\n        ssh;\n    }\n}\n",
        ))
        .unwrap()
        .services
        .unwrap();
        assert!(services.flow_tap_dtcp.is_none());
    }
}
