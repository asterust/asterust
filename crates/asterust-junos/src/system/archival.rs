//! `archival`
//! `system { archival { ... } }`.
//!
//! # See also
//!
//! - [archival](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/archival-edit-system.html)

use crate::{arg_of, child_block, has_leaf, statements, unquote};
use asterust::config::ConfigLine;

/// The `configuration` block of `system archival`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ArchivalConfiguration {
    /// Archive site URLs, with any password embedded in the URL removed. The `password` option is
    /// not extracted.
    pub archive_sites: Vec<String>,
    /// Minutes between transfers.
    pub transfer_interval: Option<String>,
    pub transfer_on_commit: bool,
}

/// The `system archival` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Archival {
    pub configuration: Option<ArchivalConfiguration>,
    pub routing_instance: Option<String>,
}

pub(super) fn parse_archival(archival: &ConfigLine) -> Archival {
    Archival {
        configuration: child_block(archival, "configuration").map(|configuration| ArchivalConfiguration {
            archive_sites: statements(configuration, "archive-sites")
                .into_iter()
                .filter_map(|site| site.split_whitespace().next())
                .map(|url| redact_password(unquote(url)))
                .collect(),
            transfer_interval: arg_of(configuration, "transfer-interval").map(str::to_string),
            transfer_on_commit: has_leaf(configuration, "transfer-on-commit"),
        }),
        routing_instance: arg_of(archival, "routing-instance").map(str::to_string),
    }
}

/// Drops the `:password` from `scheme://user:password@host/path`.
pub(super) fn redact_password(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return url.to_string();
    };
    let authority_end = rest.find('/').unwrap_or(rest.len());
    let (authority, path) = rest.split_at(authority_end);
    let Some((userinfo, host)) = authority.rsplit_once('@') else {
        return url.to_string();
    };
    let user = userinfo.split_once(':').map_or(userinfo, |(user, _)| user);
    format!("{scheme}://{user}@{host}{path}")
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax page. See the README "Sources".
    const ARCHIVAL: &str = r#"
system {
    archival {
        configuration {
            transfer-interval 60;
            transfer-on-commit;
            archive-sites {
                "scp://backup@192.0.2.9/configs" password "$9$secret"; ## SECRET-DATA
                "ftp://svc:hunter2@192.0.2.10/configs" password "$9$other"; ## SECRET-DATA
                file:///var/tmp/archive;
            }
        }
        routing-instance mgmt_junos;
    }
}
"#;

    #[test]
    fn extracts_archival_settings() {
        let archival = system(&juniper::parse_braces(ARCHIVAL)).unwrap().archival.unwrap();
        assert_eq!(archival.routing_instance.as_deref(), Some("mgmt_junos"));
        let configuration = archival.configuration.unwrap();
        assert_eq!(configuration.transfer_interval.as_deref(), Some("60"));
        assert!(configuration.transfer_on_commit);
    }

    #[test]
    fn archive_sites_never_keep_passwords() {
        let archival = system(&juniper::parse_braces(ARCHIVAL)).unwrap().archival.unwrap();
        assert_eq!(
            archival.configuration.unwrap().archive_sites,
            [
                "scp://backup@192.0.2.9/configs",
                "ftp://svc@192.0.2.10/configs",
                "file:///var/tmp/archive",
            ]
        );
    }

    #[test]
    fn archival_without_configuration() {
        let archival = system(&juniper::parse_braces(
            "system {\n    archival {\n        routing-instance mgmt_junos;\n    }\n}\n",
        ))
        .unwrap()
        .archival
        .unwrap();
        assert!(archival.configuration.is_none());
    }

    #[test]
    fn no_archival_is_none() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.archival.is_none());
    }
}
