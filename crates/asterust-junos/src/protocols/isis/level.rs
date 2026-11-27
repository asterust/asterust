//! `level`
//! `protocols { isis { level <1|2> { ... } } }`.
//!
//! # See also
//!
//! - [level (Global IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/level-edit-protocols-isis.html)
//! - [prefix-export-limit (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/prefix-export-limit-edit-protocols-isis.html)
//! - [max-lsp-size](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/max-lsp-size-edit-protocols-isis.html)
//! - [wide-metrics-only](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/wide-metrics-only-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [authentication-key (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/authentication-key-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [authentication-type (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/authentication-type-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [authentication-key-chain (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/authentication-key-chain-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [no-authentication-check](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/no-authentication-check-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [no-csnp-authentication](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/no-csnp-authentication-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [no-psnp-authentication](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/no-psnp-authentication-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [no-hello-authentication](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/no-hello-authentication-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [preference (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/preference-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [external-preference (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/external-preference-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [disable (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/disable-edit-protocols-isis.html), from the Junos 12.3 archive

use crate::{active_children, split_name, value_after};
use asterust::config::ConfigLine;

/// A `level <1|2>` entry directly under `isis`, written `level 2 wide-metrics-only;` or as a block. The
/// `authentication-key` is never extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct IsisLevel {
    /// `1` or `2`.
    pub level: String,
    pub disable: bool,
    pub wide_metrics_only: bool,
    pub preference: Option<String>,
    pub external_preference: Option<String>,
    pub prefix_export_limit: Option<String>,
    pub max_lsp_size: Option<String>,
    /// `md5` or `simple`.
    pub authentication_type: Option<String>,
    /// An `authentication-key` is configured.
    pub has_authentication_key: bool,
    pub authentication_key_chain: Option<String>,
    pub no_authentication_check: bool,
    pub no_csnp_authentication: bool,
    pub no_hello_authentication: bool,
    pub no_psnp_authentication: bool,
    /// `empty` or `self`.
    pub purge_originator: Option<String>,
}

/// `arg` is the text after `level`, e.g. `2 wide-metrics-only` or `1`.
pub(super) fn parse_level(arg: &str, level: &ConfigLine) -> IsisLevel {
    let (number, rest) = split_name(arg);
    let words: Vec<&str> = rest
        .split_whitespace()
        .chain(active_children(level).flat_map(|child| child.text.split_whitespace()))
        .collect();
    IsisLevel {
        level: number.to_string(),
        disable: words.contains(&"disable"),
        wide_metrics_only: words.contains(&"wide-metrics-only"),
        preference: value_after(&words, "preference"),
        external_preference: value_after(&words, "external-preference"),
        prefix_export_limit: value_after(&words, "prefix-export-limit"),
        max_lsp_size: value_after(&words, "max-lsp-size"),
        authentication_type: value_after(&words, "authentication-type"),
        has_authentication_key: words.contains(&"authentication-key"),
        authentication_key_chain: value_after(&words, "authentication-key-chain"),
        no_authentication_check: words.contains(&"no-authentication-check"),
        no_csnp_authentication: words.contains(&"no-csnp-authentication"),
        no_hello_authentication: words.contains(&"no-hello-authentication"),
        no_psnp_authentication: words.contains(&"no-psnp-authentication"),
        purge_originator: value_after(&words, "purge-originator"),
    }
}

#[cfg(test)]
mod tests {
    use crate::protocols::isis::isis;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    isis {
        level 1 disable;
        level 2 wide-metrics-only;
        level 2 {
            preference 20;
            external-preference 170;
            prefix-export-limit 500;
            max-lsp-size 1492;
            authentication-key "$9$secret"; ## SECRET-DATA
            authentication-type md5;
            no-hello-authentication;
            purge-originator self;
        }
    }
}
"#;

    #[test]
    fn reads_the_one_line_forms() {
        let isis = isis(&juniper::parse_braces(CONFIG)).unwrap();
        assert!(isis.levels[0].disable);
        assert_eq!(isis.levels[0].level, "1");
        assert!(isis.levels[1].wide_metrics_only);
        assert!(!isis.levels[1].disable);
    }

    #[test]
    fn reads_the_block_form_without_the_key() {
        let isis = isis(&juniper::parse_braces(CONFIG)).unwrap();
        let level = &isis.levels[2];
        assert_eq!(level.preference.as_deref(), Some("20"));
        assert_eq!(level.external_preference.as_deref(), Some("170"));
        assert_eq!(level.prefix_export_limit.as_deref(), Some("500"));
        assert_eq!(level.max_lsp_size.as_deref(), Some("1492"));
        assert_eq!(level.authentication_type.as_deref(), Some("md5"));
        assert!(level.has_authentication_key);
        assert_eq!(level.authentication_key_chain, None);
        assert!(level.no_hello_authentication);
        assert!(!level.no_csnp_authentication);
        assert_eq!(level.purge_originator.as_deref(), Some("self"));
    }
}
