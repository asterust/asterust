//! Extracts the `web-management` settings of `system services`.
//!
//! Configuration path: `system { services { web-management { ... } } }`.
//!
//! # See also
//!
//! - [web-management (System Services)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/web-management-edit-system.html)

use crate::{arg_of, args_of, child_block, has_leaf, list_values, statement_words, value_after};
use asterust::config::ConfigLine;

/// The `web-management http` block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct WebManagementHttp {
    pub interfaces: Vec<String>,
    pub port: Option<String>,
}

/// The `web-management https` block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct WebManagementHttps {
    pub interfaces: Vec<String>,
    pub port: Option<String>,
    pub local_certificate: Option<String>,
    pub pki_local_certificate: Option<String>,
    pub system_generated_certificate: bool,
}

/// The `system services web-management` settings.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct WebManagement {
    /// From `control max-threads`.
    pub control_max_threads: Option<String>,
    pub http: Option<WebManagementHttp>,
    pub https: Option<WebManagementHttps>,
    pub management_url: Option<String>,
    /// Minutes, from `session idle-timeout`.
    pub session_idle_timeout: Option<String>,
    pub session_limit: Option<String>,
}

pub(super) fn parse_web_management(web: &ConfigLine) -> WebManagement {
    let interfaces =
        |block: &ConfigLine| -> Vec<String> { args_of(block, "interface").into_iter().flat_map(list_values).collect() };
    let session = child_block(web, "session");
    WebManagement {
        control_max_threads: statement_words(web, "control").and_then(|words| value_after(&words, "max-threads")),
        http: child_block(web, "http").map(|http| WebManagementHttp {
            interfaces: interfaces(http),
            port: arg_of(http, "port").map(str::to_string),
        }),
        https: child_block(web, "https").map(|https| WebManagementHttps {
            interfaces: interfaces(https),
            port: arg_of(https, "port").map(str::to_string),
            local_certificate: arg_of(https, "local-certificate").map(str::to_string),
            pki_local_certificate: arg_of(https, "pki-local-certificate").map(str::to_string),
            system_generated_certificate: has_leaf(https, "system-generated-certificate"),
        }),
        management_url: arg_of(web, "management-url").map(str::to_string),
        session_idle_timeout: session.and_then(|s| arg_of(s, "idle-timeout")).map(str::to_string),
        session_limit: session.and_then(|s| arg_of(s, "session-limit")).map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    #[test]
    fn extracts_web_management_details() {
        let web = system(&juniper::parse_braces(
            r#"
system {
    services {
        web-management {
            control {
                max-threads 8;
            }
            management-url /manage;
            http {
                interface [ fxp0.0 ge-0/0/1.0 ];
                port 8080;
            }
            https {
                interface fxp0.0;
                port 8443;
                pki-local-certificate J-WEB;
            }
            session {
                idle-timeout 20;
                session-limit 4;
            }
        }
    }
}
"#,
        ))
        .unwrap()
        .services
        .unwrap()
        .web_management
        .unwrap();
        assert_eq!(web.control_max_threads.as_deref(), Some("8"));
        assert_eq!(web.management_url.as_deref(), Some("/manage"));
        assert_eq!(web.session_idle_timeout.as_deref(), Some("20"));
        assert_eq!(web.session_limit.as_deref(), Some("4"));

        let http = web.http.unwrap();
        assert_eq!(http.interfaces, ["fxp0.0", "ge-0/0/1.0"]);
        assert_eq!(http.port.as_deref(), Some("8080"));

        let https = web.https.unwrap();
        assert_eq!(https.interfaces, ["fxp0.0"]);
        assert_eq!(https.port.as_deref(), Some("8443"));
        assert_eq!(https.pki_local_certificate.as_deref(), Some("J-WEB"));
        assert_eq!(https.local_certificate, None);
        assert!(!https.system_generated_certificate);
    }

    #[test]
    fn web_management_with_system_generated_certificate() {
        let web = system(&juniper::parse_braces(
            "system {\n    services {\n        web-management {\n            https {\n                system-generated-certificate;\n            }\n        }\n    }\n}\n",
        ))
        .unwrap()
        .services
        .unwrap()
        .web_management
        .unwrap();
        assert!(web.https.unwrap().system_generated_certificate);
        assert!(web.http.is_none());
        assert_eq!(web.control_max_threads, None);
    }
}
