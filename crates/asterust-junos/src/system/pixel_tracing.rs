//! Extracts the `pixel-tracing` settings of `system`.
//!
//! Configuration path: `system { pixel-tracing { ... } }`.
//!
//! # See also
//!
//! - [pixel-tracing](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/pixel-tracing-edit-system.html)

use crate::arg_of;
use asterust::config::ConfigLine;

/// The `system pixel-tracing` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PixelTracing {
    /// Maximum number of dump files.
    pub files: Option<String>,
    /// `any`, `critical`, `debug`, `info` or `none`.
    pub level: Option<String>,
}

pub(super) fn parse_pixel_tracing(tracing: &ConfigLine) -> PixelTracing {
    PixelTracing {
        files: arg_of(tracing, "files").map(str::to_string),
        level: arg_of(tracing, "level").map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax page. See the README "Sources".
    #[test]
    fn extracts_pixel_tracing() {
        let tracing = system(&juniper::parse_braces(
            "system {\n    pixel-tracing {\n        files 50;\n        level info;\n    }\n}\n",
        ))
        .unwrap()
        .pixel_tracing
        .unwrap();
        assert_eq!(tracing.files.as_deref(), Some("50"));
        assert_eq!(tracing.level.as_deref(), Some("info"));
    }

    #[test]
    fn no_pixel_tracing_is_none() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.pixel_tracing.is_none());
    }
}
