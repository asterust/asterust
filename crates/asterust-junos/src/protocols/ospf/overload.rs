//! `overload`
//! `protocols { ospf { overload { ... } } }`.
//!
//! # See also
//!
//! - [overload (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/overload-edit-protocols-ospf.html)
//! - [allow-route-leaking](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/protocols-ospf-overload-allow-route-leaking.html)
//! - [as-external](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/protocols-ospf-overload-as-external.html)
//! - [stub-network](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/protocols-ospf-overload-stub-network.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The `overload` statement. A bare `overload;` leaves every field unset.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfOverload {
    /// Seconds; without it the overload bit stays set until the statement is removed.
    pub timeout: Option<String>,
    pub allow_route_leaking: bool,
    pub as_external: bool,
    pub stub_network: bool,
}

pub(super) fn parse_overload(ospf: &ConfigLine) -> Option<OspfOverload> {
    let words = statement_words(ospf, "overload")?;
    Some(OspfOverload {
        timeout: value_after(&words, "timeout"),
        allow_route_leaking: words.contains(&"allow-route-leaking"),
        as_external: words.contains(&"as-external"),
        stub_network: words.contains(&"stub-network"),
    })
}

#[cfg(test)]
mod tests {
    use crate::protocols::ospf::ospf;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const FULL: &str = r#"
protocols {
    ospf {
        overload {
            timeout 300;
            allow-route-leaking;
            as-external;
            stub-network;
        }
    }
}
"#;

    #[test]
    fn reads_the_timeout_and_flags() {
        let overload = ospf(&juniper::parse_braces(FULL)).unwrap().overload.unwrap();
        assert_eq!(overload.timeout.as_deref(), Some("300"));
        assert!(overload.allow_route_leaking);
        assert!(overload.as_external);
        assert!(overload.stub_network);
    }

    #[test]
    fn reads_the_bare_form_and_absence() {
        let bare = juniper::parse_braces("protocols {\n    ospf {\n        overload;\n    }\n}\n");
        let overload = ospf(&bare).unwrap().overload.unwrap();
        assert_eq!(overload.timeout, None);
        assert!(!overload.as_external);

        let none = juniper::parse_braces("protocols {\n    ospf {\n        area 0.0.0.0;\n    }\n}\n");
        assert_eq!(ospf(&none).unwrap().overload, None);
    }
}
