//! `schema`
//! `system { schema { ... } }`.
//!
//! # See also
//!
//! - [schema](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/schema-edit-system.html)

use crate::{child_block, has_leaf};
use asterust::config::ConfigLine;

/// The `system schema` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Schema {
    /// `openconfig { unhide; }`.
    pub openconfig_unhide: bool,
}

pub(super) fn parse_schema(schema: &ConfigLine) -> Schema {
    Schema {
        openconfig_unhide: child_block(schema, "openconfig").is_some_and(|openconfig| has_leaf(openconfig, "unhide")),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax page. See the README "Sources".
    #[test]
    fn extracts_schema_openconfig_unhide() {
        let schema = system(&juniper::parse_braces(
            "system {\n    schema {\n        openconfig {\n            unhide;\n        }\n    }\n}\n",
        ))
        .unwrap()
        .schema
        .unwrap();
        assert!(schema.openconfig_unhide);
    }

    #[test]
    fn schema_without_openconfig_unhide() {
        let schema = system(&juniper::parse_braces(
            "system {\n    schema {\n        openconfig;\n    }\n}\n",
        ))
        .unwrap()
        .schema
        .unwrap();
        assert!(!schema.openconfig_unhide);
    }

    #[test]
    fn no_schema_is_none() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.schema.is_none());
    }
}
