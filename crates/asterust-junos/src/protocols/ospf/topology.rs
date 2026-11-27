//! `topology`
//! `protocols { ospf { topology <default | ipv4-multicast | name> { ... } } }`.
//!
//! # See also
//!
//! - [topology (OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/topology-edit-protocols-ospf.html)
//! - [topology-id](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/topology-id-edit-protocols-ospf.html)

use super::backup_spf_options::{parse_backup_spf_options, OspfBackupSpfOptions};
use super::overload::{parse_overload, OspfOverload};
use super::spf_options::{parse_spf_options, OspfSpfOptions};
use crate::arg_of;
use asterust::config::ConfigLine;

/// A `topology <name>` of the OSPF instance, for multitopology routing.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfTopology {
    /// `default`, `ipv4-multicast` or a name defined under `routing-options`.
    pub name: String,
    pub topology_id: Option<String>,
    pub spf_options: Option<OspfSpfOptions>,
    pub overload: Option<OspfOverload>,
    pub prefix_export_limit: Option<String>,
    pub backup_spf_options: Option<OspfBackupSpfOptions>,
}

pub(super) fn parse_topology(name: &str, topology: &ConfigLine) -> OspfTopology {
    OspfTopology {
        name: name.to_string(),
        topology_id: arg_of(topology, "topology-id").map(str::to_string),
        spf_options: parse_spf_options(topology),
        overload: parse_overload(topology),
        prefix_export_limit: arg_of(topology, "prefix-export-limit").map(str::to_string),
        backup_spf_options: parse_backup_spf_options(topology),
    }
}

#[cfg(test)]
mod tests {
    use crate::protocols::ospf::ospf;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    ospf {
        topology VOICE {
            topology-id 50;
            prefix-export-limit 1000;
            overload {
                timeout 120;
            }
            spf-options {
                delay 100;
                rapid-runs 5;
            }
        }
        topology ipv4-multicast;
    }
}
"#;

    #[test]
    fn reads_the_topology_settings() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        assert_eq!(ospf.topologies.len(), 2);
        let voice = &ospf.topologies[0];
        assert_eq!(voice.name, "VOICE");
        assert_eq!(voice.topology_id.as_deref(), Some("50"));
        assert_eq!(voice.prefix_export_limit.as_deref(), Some("1000"));
        assert_eq!(voice.overload.as_ref().unwrap().timeout.as_deref(), Some("120"));
        let spf = voice.spf_options.as_ref().unwrap();
        assert_eq!(spf.delay.as_deref(), Some("100"));
        assert_eq!(spf.rapid_runs.as_deref(), Some("5"));
    }

    #[test]
    fn reads_a_bare_topology() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let multicast = &ospf.topologies[1];
        assert_eq!(multicast.name, "ipv4-multicast");
        assert_eq!(multicast.topology_id, None);
        assert_eq!(multicast.spf_options, None);
        assert_eq!(multicast.backup_spf_options, None);
    }
}
