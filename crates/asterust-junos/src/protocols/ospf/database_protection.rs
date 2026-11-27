//! `database-protection`
//! `protocols { ospf { database-protection { ... } } }`.
//!
//! # See also
//!
//! - [database-protection](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/database-protection-edit-protocols-ospf.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The OSPF `database-protection` limits on the LSAs a router accepts. Values are kept as written.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfDatabaseProtection {
    pub maximum_lsa: Option<String>,
    pub ignore_count: Option<String>,
    /// Seconds.
    pub ignore_time: Option<String>,
    /// Seconds.
    pub reset_time: Option<String>,
    /// Percent.
    pub warning_threshold: Option<String>,
    pub warning_only: bool,
}

pub(super) fn parse_database_protection(ospf: &ConfigLine) -> Option<OspfDatabaseProtection> {
    let words = statement_words(ospf, "database-protection")?;
    Some(OspfDatabaseProtection {
        maximum_lsa: value_after(&words, "maximum-lsa"),
        ignore_count: value_after(&words, "ignore-count"),
        ignore_time: value_after(&words, "ignore-time"),
        reset_time: value_after(&words, "reset-time"),
        warning_threshold: value_after(&words, "warning-threshold"),
        warning_only: words.contains(&"warning-only"),
    })
}

#[cfg(test)]
mod tests {
    use crate::protocols::ospf::ospf;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    ospf {
        database-protection {
            maximum-lsa 100000;
            ignore-count 3;
            ignore-time 120;
            reset-time 900;
            warning-threshold 80;
            warning-only;
        }
    }
}
"#;

    #[test]
    fn reads_the_limits() {
        let protection = ospf(&juniper::parse_braces(CONFIG))
            .unwrap()
            .database_protection
            .unwrap();
        assert_eq!(protection.maximum_lsa.as_deref(), Some("100000"));
        assert_eq!(protection.ignore_count.as_deref(), Some("3"));
        assert_eq!(protection.ignore_time.as_deref(), Some("120"));
        assert_eq!(protection.reset_time.as_deref(), Some("900"));
        assert_eq!(protection.warning_threshold.as_deref(), Some("80"));
        assert!(protection.warning_only);
    }

    #[test]
    fn leaves_unwritten_limits_unset() {
        let config = juniper::parse_braces("protocols {\n    ospf {\n        database-protection {\n            maximum-lsa 500;\n        }\n    }\n}\n");
        let protection = ospf(&config).unwrap().database_protection.unwrap();
        assert_eq!(protection.maximum_lsa.as_deref(), Some("500"));
        assert_eq!(protection.reset_time, None);
        assert!(!protection.warning_only);
    }
}
