//! `system { dynamic-profile-options { ... } }`.

use crate::has_leaf;
use asterust::config::ConfigLine;

/// The `system dynamic-profile-options` settings.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DynamicProfileOptions {
    pub versioning: bool,
}

pub(super) fn parse_dynamic_profile_options(options: &ConfigLine) -> DynamicProfileOptions {
    DynamicProfileOptions {
        versioning: has_leaf(options, "versioning"),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_dynamic_profile_options() {
        let system = system(&juniper::parse_braces(
            "system {\n    dynamic-profile-options {\n        versioning;\n    }\n}\n",
        ))
        .unwrap();
        assert!(system.dynamic_profile_options.unwrap().versioning);
    }
}
