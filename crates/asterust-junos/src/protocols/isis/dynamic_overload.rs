//! `dynamic-overload`
//! `protocols { isis { dynamic-overload { auto-recovery; no-overload-on-prefix-export-limit; } } }`.
//!
//! # See also
//!
//! - [auto-recovery](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/protocols-isis-dynamic-overload-auto-recovery.html)
//! - [no-overload-on-prefix-export-limit](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/protocols-isis-dynamic-overload-no-overload-on-prefix-export-limit.html)

use crate::statement_words;
use asterust::config::ConfigLine;

/// The IS-IS `dynamic-overload` statement. It has no page of its own; its children are documented separately.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct IsisDynamicOverload {
    pub auto_recovery: bool,
    pub no_overload_on_prefix_export_limit: bool,
}

pub(super) fn parse_dynamic_overload(isis: &ConfigLine) -> Option<IsisDynamicOverload> {
    let words = statement_words(isis, "dynamic-overload")?;
    Some(IsisDynamicOverload {
        auto_recovery: words.contains(&"auto-recovery"),
        no_overload_on_prefix_export_limit: words.contains(&"no-overload-on-prefix-export-limit"),
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
        dynamic-overload {
            auto-recovery;
            no-overload-on-prefix-export-limit;
        }
    }
}
"#;

    #[test]
    fn reads_both_flags() {
        let dynamic = isis(&juniper::parse_braces(CONFIG)).unwrap().dynamic_overload.unwrap();
        assert!(dynamic.auto_recovery);
        assert!(dynamic.no_overload_on_prefix_export_limit);
    }

    #[test]
    fn reads_one_flag_and_absence() {
        let config = juniper::parse_braces(
            "protocols {\n    isis {\n        dynamic-overload {\n            auto-recovery;\n        }\n    }\n}\n",
        );
        let dynamic = isis(&config).unwrap().dynamic_overload.unwrap();
        assert!(dynamic.auto_recovery);
        assert!(!dynamic.no_overload_on_prefix_export_limit);

        let none = juniper::parse_braces("protocols {\n    isis {\n        interface ge-0/0/0.0;\n    }\n}\n");
        assert_eq!(isis(&none).unwrap().dynamic_overload, None);
    }
}
