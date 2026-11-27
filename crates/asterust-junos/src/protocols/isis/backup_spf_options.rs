//! `backup-spf-options`
//! `protocols { isis { backup-spf-options { ... } } }`.
//!
//! # See also
//!
//! - [backup-spf-options (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/backup-spf-options-edit-protocols-isis.html)
//! - [use-post-convergence-lfa](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/protocols-isis-backup-spf-options-use-post-convergence-lfa.html)

use crate::{child_block, statement_words, value_after};
use asterust::config::ConfigLine;

/// The IS-IS `backup-spf-options`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct IsisBackupSpfOptions {
    pub node_link_degradation: bool,
    pub per_prefix_calculation: bool,
    pub remote_backup_calculation: bool,
    pub use_source_packet_routing: bool,
    /// A `use-post-convergence-lfa` statement is configured.
    pub use_post_convergence_lfa: bool,
    pub maximum_backup_paths: Option<String>,
    pub maximum_labels: Option<String>,
}

pub(super) fn parse_backup_spf_options(isis: &ConfigLine) -> Option<IsisBackupSpfOptions> {
    let words = statement_words(isis, "backup-spf-options")?;
    let lfa =
        child_block(isis, "backup-spf-options").and_then(|backup| statement_words(backup, "use-post-convergence-lfa"));
    Some(IsisBackupSpfOptions {
        node_link_degradation: words.contains(&"node-link-degradation"),
        per_prefix_calculation: words.contains(&"per-prefix-calculation"),
        remote_backup_calculation: words.contains(&"remote-backup-calculation"),
        use_source_packet_routing: words.contains(&"use-source-packet-routing"),
        use_post_convergence_lfa: lfa.is_some(),
        maximum_backup_paths: lfa.as_ref().and_then(|lfa| value_after(lfa, "maximum-backup-paths")),
        maximum_labels: lfa.as_ref().and_then(|lfa| value_after(lfa, "maximum-labels")),
    })
}

#[cfg(test)]
mod tests {
    use crate::protocols::isis::isis;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    isis {
        backup-spf-options {
            node-link-degradation;
            per-prefix-calculation;
            remote-backup-calculation;
            use-source-packet-routing;
            use-post-convergence-lfa {
                maximum-backup-paths 8;
                maximum-labels 5;
            }
        }
    }
}
"#;

    #[test]
    fn reads_the_flags_and_post_convergence_lfa() {
        let backup = isis(&juniper::parse_braces(CONFIG))
            .unwrap()
            .backup_spf_options
            .unwrap();
        assert!(backup.node_link_degradation);
        assert!(backup.per_prefix_calculation);
        assert!(backup.remote_backup_calculation);
        assert!(backup.use_source_packet_routing);
        assert!(backup.use_post_convergence_lfa);
        assert_eq!(backup.maximum_backup_paths.as_deref(), Some("8"));
        assert_eq!(backup.maximum_labels.as_deref(), Some("5"));
    }

    #[test]
    fn reads_the_inline_lfa_form_and_absence() {
        let config = juniper::parse_braces(
            "protocols {\n    isis {\n        backup-spf-options {\n            use-post-convergence-lfa maximum-labels 4;\n        }\n    }\n}\n",
        );
        let backup = isis(&config).unwrap().backup_spf_options.unwrap();
        assert!(backup.use_post_convergence_lfa);
        assert_eq!(backup.maximum_labels.as_deref(), Some("4"));
        assert_eq!(backup.maximum_backup_paths, None);
        assert!(!backup.node_link_degradation);
    }
}
