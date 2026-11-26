//! `system { ... }`.

use crate::{arg_of, top_level};
use asterust::config::Config;

/// The top-level `system` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct System {
    pub host_name: Option<String>,
}

/// The top-level `system` block, or `None` if there is none.
pub fn system(config: &Config) -> Option<System> {
    let system = top_level(config, "system")?;
    Some(System {
        host_name: arg_of(system, "host-name").map(str::to_string),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_host_name() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert_eq!(system.host_name.as_deref(), Some("pe1"));
    }

    #[test]
    fn host_name_is_optional() {
        let system = system(&juniper::parse_braces("system {\n    domain-name example.net;\n}\n")).unwrap();
        assert_eq!(system.host_name, None);
    }

    #[test]
    fn skips_inactive_host_name() {
        let system = system(&juniper::parse_braces("system {\n    inactive: host-name pe1;\n}\n")).unwrap();
        assert_eq!(system.host_name, None);
    }

    #[test]
    fn no_system_is_none() {
        assert!(system(&juniper::parse_braces("interfaces {\n    lo0;\n}\n")).is_none());
    }
}
