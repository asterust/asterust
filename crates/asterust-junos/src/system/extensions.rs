//! Extracts the `extensions` settings of `system`.
//!
//! Configuration path: `system { extensions { ... } }`.
//!
//! # See also
//!
//! - [extensions](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/extensions-edit-system.html)

use crate::{active_children, arg_of, args_of, child_block, has_leaf, list_values, named_children, split_name};
use asterust::config::ConfigLine;

/// A `file <name>` of `extension-service application`. `cert-file`, `checksum`, `source`, `refresh`
/// and `start` options are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ExtensionApplication {
    pub name: String,
    pub arguments: Option<String>,
    pub daemonize: bool,
    /// `bash`, `python` or `python3`.
    pub interpreter: Option<String>,
    pub optional: bool,
    pub routing_instance: Option<String>,
    pub username: Option<String>,
}

/// A `license-type <license> deployment-scope [ ... ]` line of a provider.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ProviderLicense {
    pub license_type: String,
    pub deployment_scopes: Vec<String>,
}

/// A provider under `extensions providers`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ExtensionProvider {
    pub id: String,
    pub licenses: Vec<ProviderLicense>,
}

/// The `system extensions` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Extensions {
    /// Files from `extension-service { application { file ... } }`.
    pub applications: Vec<ExtensionApplication>,
    pub max_datasize: Option<String>,
    pub providers: Vec<ExtensionProvider>,
}

pub(super) fn parse_extensions(extensions: &ConfigLine) -> Extensions {
    let application =
        child_block(extensions, "extension-service").and_then(|service| child_block(service, "application"));
    Extensions {
        applications: application
            .map(|application| {
                named_children(application, "file")
                    .map(|(name, file)| ExtensionApplication {
                        name: name.to_string(),
                        arguments: arg_of(file, "arguments").map(str::to_string),
                        daemonize: has_leaf(file, "daemonize"),
                        interpreter: arg_of(file, "interpreter").map(str::to_string),
                        optional: has_leaf(file, "optional"),
                        routing_instance: arg_of(file, "routing-instance").map(str::to_string),
                        username: arg_of(file, "username").map(str::to_string),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        max_datasize: application
            .and_then(|application| arg_of(application, "max-datasize"))
            .map(str::to_string),
        providers: child_block(extensions, "providers")
            .map(|providers| {
                active_children(providers)
                    .map(|provider| ExtensionProvider {
                        id: provider.text.clone(),
                        licenses: args_of(provider, "license-type")
                            .into_iter()
                            .map(|arg| {
                                let (license_type, rest) = split_name(arg);
                                ProviderLicense {
                                    license_type: license_type.to_string(),
                                    deployment_scopes: rest
                                        .strip_prefix("deployment-scope")
                                        .map(list_values)
                                        .unwrap_or_default(),
                                }
                            })
                            .collect(),
                    })
                    .collect()
            })
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax page. See the README "Sources".
    const EXTENSIONS: &str = r#"
system {
    extensions {
        extension-service {
            application {
                file monitor.py {
                    arguments "--verbose";
                    daemonize;
                    interpreter python3;
                    optional;
                    routing-instance mgmt_junos;
                    username jet;
                    checksum sha-256 abc123;
                }
                file plain.sh;
                max-datasize 64m;
            }
        }
        providers {
            ISV1 {
                license-type juniper deployment-scope [ commercial ];
                license-type customer deployment-scope [ poc commercial ];
            }
        }
    }
}
"#;

    #[test]
    fn extracts_extension_applications() {
        let extensions = system(&juniper::parse_braces(EXTENSIONS)).unwrap().extensions.unwrap();
        assert_eq!(extensions.max_datasize.as_deref(), Some("64m"));
        let names: Vec<_> = extensions.applications.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["monitor.py", "plain.sh"]);

        let monitor = &extensions.applications[0];
        assert_eq!(monitor.arguments.as_deref(), Some("--verbose"));
        assert!(monitor.daemonize);
        assert!(monitor.optional);
        assert_eq!(monitor.interpreter.as_deref(), Some("python3"));
        assert_eq!(monitor.routing_instance.as_deref(), Some("mgmt_junos"));
        assert_eq!(monitor.username.as_deref(), Some("jet"));

        let plain = &extensions.applications[1];
        assert!(!plain.daemonize);
        assert_eq!(plain.interpreter, None);
    }

    #[test]
    fn extracts_extension_providers() {
        let extensions = system(&juniper::parse_braces(EXTENSIONS)).unwrap().extensions.unwrap();
        assert_eq!(extensions.providers.len(), 1);
        let provider = &extensions.providers[0];
        assert_eq!(provider.id, "ISV1");
        assert_eq!(provider.licenses.len(), 2);
        assert_eq!(provider.licenses[0].license_type, "juniper");
        assert_eq!(provider.licenses[0].deployment_scopes, ["commercial"]);
        assert_eq!(provider.licenses[1].deployment_scopes, ["poc", "commercial"]);
    }

    #[test]
    fn extensions_with_only_providers() {
        let extensions = system(&juniper::parse_braces(
            "system {\n    extensions {\n        providers {\n            ISV2;\n        }\n    }\n}\n",
        ))
        .unwrap()
        .extensions
        .unwrap();
        assert!(extensions.applications.is_empty());
        assert_eq!(extensions.max_datasize, None);
        assert_eq!(extensions.providers[0].id, "ISV2");
        assert!(extensions.providers[0].licenses.is_empty());
    }

    #[test]
    fn no_extensions_is_none() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.extensions.is_none());
    }
}
