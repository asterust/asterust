//! `authentication`
//! `protocols { ospf { area <id> { interface <name> { authentication { ... } } } } }`.
//!
//! # See also
//!
//! - [authentication (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/authentication-edit-protocols-ospf.html)

use crate::{arg_of, child_block, named_children, split_name};
use asterust::config::ConfigLine;

/// The `authentication` of an OSPF interface. The `simple-password` and the MD5 key values are never extracted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfAuthentication {
    pub simple_password: bool,
    pub md5: Vec<OspfMd5Key>,
    /// Key identifiers of `multi-active-md5`.
    pub multi_active_md5: Vec<String>,
    /// The name of the key chain, from `keychain` (23.4 and later).
    pub keychain: Option<String>,
}

/// An `md5 <key-id>` entry; only the identifier and the start time are kept.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfMd5Key {
    pub id: String,
    pub start_time: Option<String>,
}

pub(super) fn parse_authentication(iface: &ConfigLine) -> Option<OspfAuthentication> {
    let auth = child_block(iface, "authentication")?;
    Some(OspfAuthentication {
        simple_password: arg_of(auth, "simple-password").is_some(),
        md5: named_children(auth, "md5")
            .map(|(arg, key)| OspfMd5Key {
                id: split_name(arg).0.to_string(),
                start_time: arg_of(key, "start-time").map(str::to_string),
            })
            .collect(),
        multi_active_md5: named_children(auth, "multi-active-md5")
            .map(|(arg, _)| split_name(arg).0.to_string())
            .collect(),
        keychain: arg_of(auth, "keychain").map(str::to_string),
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
        area 0.0.0.0 {
            interface ge-0/0/0.0 {
                authentication {
                    md5 1 {
                        key "$9$secret"; ## SECRET-DATA
                        start-time "2026-1-1.00:00";
                    }
                    md5 2 key "$9$other"; ## SECRET-DATA
                }
            }
            interface ge-0/0/1.0 {
                authentication {
                    simple-password "$9$secret"; ## SECRET-DATA
                }
            }
            interface ge-0/0/2.0 {
                authentication {
                    multi-active-md5 5 key "$9$secret" delete-if-not-inuse; ## SECRET-DATA
                    keychain OSPF-KEYS;
                }
            }
        }
    }
}
"#;

    #[test]
    fn reads_md5_key_ids_without_the_keys() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let auth = ospf.areas[0].interfaces[0].authentication.as_ref().unwrap();
        assert_eq!(auth.md5.len(), 2);
        assert_eq!(auth.md5[0].id, "1");
        assert_eq!(auth.md5[0].start_time.as_deref(), Some("2026-1-1.00:00"));
        assert_eq!(auth.md5[1].id, "2");
        assert_eq!(auth.md5[1].start_time, None);
        assert!(!auth.simple_password);
    }

    #[test]
    fn reads_simple_password_multi_active_md5_and_keychain() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let interfaces = &ospf.areas[0].interfaces;
        assert!(interfaces[1].authentication.as_ref().unwrap().simple_password);

        let multi = interfaces[2].authentication.as_ref().unwrap();
        assert_eq!(multi.multi_active_md5, ["5"]);
        assert_eq!(multi.keychain.as_deref(), Some("OSPF-KEYS"));
        assert!(multi.md5.is_empty());
    }
}
