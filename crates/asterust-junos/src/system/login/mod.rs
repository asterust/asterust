//! `system { login { ... } }`.

mod class;
mod password;
mod retry_options;
mod user;

pub use class::LoginClass;
pub use password::LoginPassword;
pub use retry_options::LoginRetryOptions;
pub use user::LoginUser;

use crate::{arg_of, args_of, child_block, list_values, named_children};
use asterust::config::ConfigLine;
use class::parse_class;
use password::parse_password;
use retry_options::parse_retry_options;
use user::parse_user;

/// The `system login` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Login {
    /// The banner shown before login.
    pub message: Option<String>,
    /// The text shown after login.
    pub announcement: Option<String>,
    /// Minutes.
    pub idle_timeout: Option<String>,
    /// Addresses from `deny-sources { address [ ... ]; }`.
    pub deny_sources: Vec<String>,
    pub password: Option<LoginPassword>,
    pub retry_options: Option<LoginRetryOptions>,
    pub classes: Vec<LoginClass>,
    pub users: Vec<LoginUser>,
}

/// Values of every `<keyword> ...` child, whether written as a bare value or as a `[ list ]`.
fn list(line: &ConfigLine, keyword: &str) -> Vec<String> {
    args_of(line, keyword).into_iter().flat_map(list_values).collect()
}

pub(super) fn parse_login(login: &ConfigLine) -> Login {
    Login {
        message: arg_of(login, "message").map(str::to_string),
        announcement: arg_of(login, "announcement").map(str::to_string),
        idle_timeout: arg_of(login, "idle-timeout").map(str::to_string),
        deny_sources: child_block(login, "deny-sources")
            .map(|sources| list(sources, "address"))
            .unwrap_or_default(),
        password: child_block(login, "password").map(parse_password),
        retry_options: child_block(login, "retry-options").map(parse_retry_options),
        classes: named_children(login, "class")
            .map(|(name, class)| parse_class(name, class))
            .collect(),
        users: named_children(login, "user")
            .map(|(name, user)| parse_user(name, user))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    const LOGIN: &str = r#"
system {
    login {
        message "Authorized access only";
        announcement "Maintenance window Sunday";
        class NOC {
            permissions [ view view-configuration ];
            idle-timeout 15;
            allow-commands "(ping)|(traceroute)";
            deny-configuration ".*";
        }
        class OPS {
            permissions all;
            deny-commands "(request system)";
            allow-configuration "(interfaces)|(protocols)";
        }
        user admin {
            uid 2000;
            class super-user;
            full-name "Network Admin";
            authentication {
                encrypted-password "$6$salt$hash"; ## SECRET-DATA
                ssh-ed25519 "ssh-ed25519 AAAAC3Nza admin@host"; ## SECRET-DATA
            }
        }
        user monitor {
            class NOC;
        }
    }
}
"#;

    #[test]
    fn extracts_login_banner_and_classes() {
        let login = system(&juniper::parse_braces(LOGIN)).unwrap().login.unwrap();
        assert_eq!(login.message.as_deref(), Some("Authorized access only"));
        assert_eq!(login.announcement.as_deref(), Some("Maintenance window Sunday"));

        assert_eq!(login.classes.len(), 2);
        let noc = &login.classes[0];
        assert_eq!(noc.name, "NOC");
        assert_eq!(noc.permissions, ["view", "view-configuration"]);
        assert_eq!(noc.idle_timeout.as_deref(), Some("15"));
        assert_eq!(noc.allow_commands.as_deref(), Some("(ping)|(traceroute)"));
        assert_eq!(noc.deny_configuration.as_deref(), Some(".*"));
        assert_eq!(noc.deny_commands, None);

        let ops = &login.classes[1];
        assert_eq!(ops.permissions, ["all"]);
        assert_eq!(ops.deny_commands.as_deref(), Some("(request system)"));
        assert_eq!(ops.allow_configuration.as_deref(), Some("(interfaces)|(protocols)"));
    }

    #[test]
    fn extracts_login_users_without_secrets() {
        let login = system(&juniper::parse_braces(LOGIN)).unwrap().login.unwrap();
        assert_eq!(login.users.len(), 2);

        let admin = &login.users[0];
        assert_eq!(admin.name, "admin");
        assert_eq!(admin.class.as_deref(), Some("super-user"));
        assert_eq!(admin.uid.as_deref(), Some("2000"));
        assert_eq!(admin.full_name.as_deref(), Some("Network Admin"));
        assert_eq!(admin.authentication_methods, ["encrypted-password", "ssh-ed25519"]);

        let monitor = &login.users[1];
        assert_eq!(monitor.class.as_deref(), Some("NOC"));
        assert!(monitor.authentication_methods.is_empty());
    }

    // Not from documentation: written from the CLI Reference syntax page. See the README "Sources".
    const LOGIN_OPTIONS: &str = r#"
system {
    login {
        idle-timeout 30;
        deny-sources {
            address [ 203.0.113.0/24 198.51.100.7 ];
        }
        password {
            format sha512;
            change-type character-sets;
            minimum-length 12;
            maximum-length 64;
            minimum-changes 3;
            minimum-upper-cases 1;
            minimum-lower-cases 1;
            minimum-numerics 1;
            minimum-punctuations 1;
            minimum-reuse 5;
            maximum-lifetime 90;
            minimum-lifetime 1;
            minimum-character-changes 2;
        }
        retry-options {
            tries-before-disconnect 3;
            backoff-threshold 2;
            backoff-factor 5;
            minimum-time 20;
            maximum-time 60;
            lockout-period 10;
        }
        class OPS {
            access-start 08:00;
            access-end 18:00;
            allowed-days [ monday tuesday friday ];
            allow-sources [ 192.0.2.0/24 ];
            deny-sources 192.0.2.99;
            allow-times [ 08:00-12:00 ];
            deny-times [ 22:00-23:00 ];
        }
    }
}
"#;

    #[test]
    fn extracts_login_level_options() {
        let login = system(&juniper::parse_braces(LOGIN_OPTIONS)).unwrap().login.unwrap();
        assert_eq!(login.idle_timeout.as_deref(), Some("30"));
        assert_eq!(login.deny_sources, ["203.0.113.0/24", "198.51.100.7"]);
    }

    #[test]
    fn extracts_password_policy() {
        let login = system(&juniper::parse_braces(LOGIN_OPTIONS)).unwrap().login.unwrap();
        let password = login.password.unwrap();
        assert_eq!(password.format.as_deref(), Some("sha512"));
        assert_eq!(password.change_type.as_deref(), Some("character-sets"));
        assert_eq!(password.minimum_length.as_deref(), Some("12"));
        assert_eq!(password.maximum_length.as_deref(), Some("64"));
        assert_eq!(password.minimum_changes.as_deref(), Some("3"));
        assert_eq!(password.minimum_upper_cases.as_deref(), Some("1"));
        assert_eq!(password.minimum_lower_cases.as_deref(), Some("1"));
        assert_eq!(password.minimum_numerics.as_deref(), Some("1"));
        assert_eq!(password.minimum_punctuations.as_deref(), Some("1"));
        assert_eq!(password.minimum_reuse.as_deref(), Some("5"));
        assert_eq!(password.maximum_lifetime.as_deref(), Some("90"));
        assert_eq!(password.minimum_lifetime.as_deref(), Some("1"));
        assert_eq!(password.minimum_character_changes.as_deref(), Some("2"));
    }

    #[test]
    fn extracts_retry_options() {
        let login = system(&juniper::parse_braces(LOGIN_OPTIONS)).unwrap().login.unwrap();
        let retry = login.retry_options.unwrap();
        assert_eq!(retry.tries_before_disconnect.as_deref(), Some("3"));
        assert_eq!(retry.backoff_threshold.as_deref(), Some("2"));
        assert_eq!(retry.backoff_factor.as_deref(), Some("5"));
        assert_eq!(retry.minimum_time.as_deref(), Some("20"));
        assert_eq!(retry.maximum_time.as_deref(), Some("60"));
        assert_eq!(retry.lockout_period.as_deref(), Some("10"));
    }

    #[test]
    fn extracts_class_access_restrictions() {
        let login = system(&juniper::parse_braces(LOGIN_OPTIONS)).unwrap().login.unwrap();
        let class = &login.classes[0];
        assert_eq!(class.access_start.as_deref(), Some("08:00"));
        assert_eq!(class.access_end.as_deref(), Some("18:00"));
        assert_eq!(class.allowed_days, ["monday", "tuesday", "friday"]);
        assert_eq!(class.allow_sources, ["192.0.2.0/24"]);
        assert_eq!(class.deny_sources, ["192.0.2.99"]);
        assert_eq!(class.allow_times, ["08:00-12:00"]);
        assert_eq!(class.deny_times, ["22:00-23:00"]);
    }

    #[test]
    fn login_options_are_optional() {
        let login = system(&juniper::parse_braces(
            "system {\n    login {\n        class OPS;\n    }\n}\n",
        ))
        .unwrap()
        .login
        .unwrap();
        assert_eq!(login.idle_timeout, None);
        assert!(login.deny_sources.is_empty());
        assert!(login.password.is_none());
        assert!(login.retry_options.is_none());
    }

    #[test]
    fn no_login_is_none() {
        assert!(system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n"))
            .unwrap()
            .login
            .is_none());
    }
}
