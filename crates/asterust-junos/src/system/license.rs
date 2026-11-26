//! `license`
//! `system { license { ... } }`.
//!
//! # See also
//!
//! - [license](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/license-edit-system.html)

use super::archival::redact_password;
use crate::{active_children, arg_of, child_block, statement_words, unquote, value_after};
use asterust::config::ConfigLine;

/// The `autoupdate` block of `system license`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LicenseAutoupdate {
    /// The license server URL, without any password embedded in it. The `password` option is not
    /// extracted.
    pub url: Option<String>,
}

/// The `renew` block of `system license`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LicenseRenew {
    /// Days before expiration to renew.
    pub before_expiration: Option<String>,
    /// Hours between license checks.
    pub interval: Option<String>,
}

/// The `system license` configuration. License keys are not extracted, only how many are
/// configured.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct License {
    pub autoupdate: Option<LicenseAutoupdate>,
    pub key_count: usize,
    pub renew: Option<LicenseRenew>,
}

pub(super) fn parse_license(license: &ConfigLine) -> License {
    License {
        autoupdate: statement_words(license, "autoupdate").map(|words| LicenseAutoupdate {
            url: value_after(&words, "url").map(|url| redact_password(unquote(&url))),
        }),
        key_count: child_block(license, "keys")
            .map(|keys| active_children(keys).filter(|k| k.arg("key").is_some()).count())
            .unwrap_or(0),
        renew: child_block(license, "renew").map(|renew| LicenseRenew {
            before_expiration: arg_of(renew, "before-expiration").map(str::to_string),
            interval: arg_of(renew, "interval").map(str::to_string),
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax page. See the README "Sources".
    const LICENSE: &str = r#"
system {
    license {
        autoupdate {
            url "https://user:hunter2@ae1.juniper.net/junos/key_retrieval" password "$9$secret"; ## SECRET-DATA
        }
        keys {
            key "JUNOS000001 aeaqcaj...";
            key "JUNOS000002 bfbrdbk...";
        }
        renew {
            before-expiration 30;
            interval 24;
        }
    }
}
"#;

    #[test]
    fn extracts_license_without_secrets() {
        let license = system(&juniper::parse_braces(LICENSE)).unwrap().license.unwrap();
        assert_eq!(
            license.autoupdate.unwrap().url.as_deref(),
            Some("https://user@ae1.juniper.net/junos/key_retrieval")
        );
        assert_eq!(license.key_count, 2);
        let renew = license.renew.unwrap();
        assert_eq!(renew.before_expiration.as_deref(), Some("30"));
        assert_eq!(renew.interval.as_deref(), Some("24"));
    }

    #[test]
    fn license_with_only_autoupdate() {
        let license = system(&juniper::parse_braces(
            "system {\n    license {\n        autoupdate {\n            url https://ae1.juniper.net/key;\n        }\n    }\n}\n",
        ))
        .unwrap()
        .license
        .unwrap();
        assert_eq!(
            license.autoupdate.unwrap().url.as_deref(),
            Some("https://ae1.juniper.net/key")
        );
        assert_eq!(license.key_count, 0);
        assert!(license.renew.is_none());
    }

    #[test]
    fn no_license_is_none() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.license.is_none());
    }
}
