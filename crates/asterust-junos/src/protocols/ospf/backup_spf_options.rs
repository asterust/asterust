//! `backup-spf-options`
//! `protocols { ospf { [topology <name> {] backup-spf-options { ... } } } }`.
//!
//! # See also
//!
//! - [backup-spf-options (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/backup-spf-options-edit-protocols-ospf.html)
//! - [use-post-convergence-lfa (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/use-post-convergence-editprotocols-ospf-backup-spf-options.html)
//! - [Configuring Segment Routing Microloop Avoidance in OSPFv2 Networks](https://networkingdocs.hpe.com/documentation/us/en/software/junos/segment-routing/topics/example/ospfv2-microloop-avoidance-configuring.html)

use crate::{child_block, statement_words, value_after};
use asterust::config::ConfigLine;

/// The `backup-spf-options` of an OSPF instance or topology.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfBackupSpfOptions {
    pub disable: bool,
    pub downstream_paths_only: bool,
    pub no_install: bool,
    pub node_link_degradation: bool,
    pub remote_backup_calculation: bool,
    pub use_source_packet_routing: bool,
    pub no_source_packet_routing: bool,
    /// `None` without `per-prefix-calculation`; otherwise the written members of `all`, `externals`, `stubs` and
    /// `summary`.
    pub per_prefix_calculation: Option<Vec<String>>,
    /// A `use-post-convergence-lfa` statement is configured.
    pub use_post_convergence_lfa: bool,
    pub maximum_backup_paths: Option<String>,
    pub maximum_labels: Option<String>,
}

pub(super) fn parse_backup_spf_options(parent: &ConfigLine) -> Option<OspfBackupSpfOptions> {
    let words = statement_words(parent, "backup-spf-options")?;
    let backup = child_block(parent, "backup-spf-options");
    let per_prefix = backup.and_then(|backup| statement_words(backup, "per-prefix-calculation"));
    let lfa = backup.and_then(|backup| statement_words(backup, "use-post-convergence-lfa"));
    Some(OspfBackupSpfOptions {
        disable: words.contains(&"disable"),
        downstream_paths_only: words.contains(&"downstream-paths-only"),
        no_install: words.contains(&"no-install"),
        node_link_degradation: words.contains(&"node-link-degradation"),
        remote_backup_calculation: words.contains(&"remote-backup-calculation"),
        use_source_packet_routing: words.contains(&"use-source-packet-routing"),
        no_source_packet_routing: words.contains(&"no-source-packet-routing"),
        per_prefix_calculation: per_prefix.map(|words| {
            words
                .into_iter()
                .filter(|word| matches!(*word, "all" | "externals" | "stubs" | "summary"))
                .map(str::to_string)
                .collect()
        }),
        use_post_convergence_lfa: lfa.is_some(),
        maximum_backup_paths: lfa.as_ref().and_then(|lfa| value_after(lfa, "maximum-backup-paths")),
        maximum_labels: lfa.as_ref().and_then(|lfa| value_after(lfa, "maximum-labels")),
    })
}

#[cfg(test)]
mod tests {
    use crate::protocols::ospf::ospf;
    use asterust::config::juniper;

    // Adapted from "Configuring Segment Routing Microloop Avoidance in OSPFv2 Networks" (Segment Routing User
    // Guide), Device R0 `show protocols` (OSPF part), plus a second block written from the CLI Reference; see the
    // README "Sources".
    const CONFIG: &str = r#"
protocols {
    ospf {
        backup-spf-options {
            use-post-convergence-lfa {
                maximum-labels 5;
                maximum-backup-paths 8;
            }
            use-source-packet-routing;
        }
        topology ipv4-multicast {
            backup-spf-options {
                disable;
                downstream-paths-only;
                no-install;
                node-link-degradation;
                remote-backup-calculation;
                per-prefix-calculation {
                    all;
                    stubs;
                }
            }
        }
    }
}
"#;

    #[test]
    fn reads_post_convergence_lfa_and_source_packet_routing() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let backup = ospf.backup_spf_options.unwrap();
        assert!(backup.use_post_convergence_lfa);
        assert_eq!(backup.maximum_labels.as_deref(), Some("5"));
        assert_eq!(backup.maximum_backup_paths.as_deref(), Some("8"));
        assert!(backup.use_source_packet_routing);
        assert!(!backup.disable);
        assert_eq!(backup.per_prefix_calculation, None);
    }

    #[test]
    fn reads_the_flags_and_per_prefix_calculation_of_a_topology() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let backup = ospf.topologies[0].backup_spf_options.as_ref().unwrap();
        assert!(backup.disable);
        assert!(backup.downstream_paths_only);
        assert!(backup.no_install);
        assert!(backup.node_link_degradation);
        assert!(backup.remote_backup_calculation);
        assert_eq!(
            backup.per_prefix_calculation,
            Some(vec!["all".to_string(), "stubs".to_string()])
        );
        assert!(!backup.use_post_convergence_lfa);
    }
}
