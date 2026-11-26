//! `system { login { ... } }`.

use super::auth::authentication_methods;
use crate::{arg_of, args_of, child_block, list_values, named_children};
use asterust::config::ConfigLine;

/// A login `class`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LoginClass {
    pub name: String,
    pub permissions: Vec<String>,
    /// Minutes.
    pub idle_timeout: Option<String>,
    /// The regular expressions as written, e.g. `"(show)|(ping)"`.
    pub allow_commands: Option<String>,
    pub deny_commands: Option<String>,
    pub allow_configuration: Option<String>,
    pub deny_configuration: Option<String>,
}

/// A login `user`. Passwords and keys are not extracted, only which authentication methods are present.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LoginUser {
    pub name: String,
    pub class: Option<String>,
    pub uid: Option<String>,
    pub full_name: Option<String>,
    /// One entry per authentication statement, e.g. `"encrypted-password"`, `"ssh-rsa"`.
    pub authentication_methods: Vec<String>,
}

/// The `system login` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Login {
    /// The banner shown before login.
    pub message: Option<String>,
    /// The text shown after login.
    pub announcement: Option<String>,
    pub classes: Vec<LoginClass>,
    pub users: Vec<LoginUser>,
}

pub(super) fn parse_login(login: &ConfigLine) -> Login {
    Login {
        message: arg_of(login, "message").map(str::to_string),
        announcement: arg_of(login, "announcement").map(str::to_string),
        classes: named_children(login, "class")
            .map(|(name, class)| LoginClass {
                name: name.to_string(),
                permissions: args_of(class, "permissions")
                    .into_iter()
                    .flat_map(list_values)
                    .collect(),
                idle_timeout: arg_of(class, "idle-timeout").map(str::to_string),
                allow_commands: arg_of(class, "allow-commands").map(str::to_string),
                deny_commands: arg_of(class, "deny-commands").map(str::to_string),
                allow_configuration: arg_of(class, "allow-configuration").map(str::to_string),
                deny_configuration: arg_of(class, "deny-configuration").map(str::to_string),
            })
            .collect(),
        users: named_children(login, "user")
            .map(|(name, user)| LoginUser {
                name: name.to_string(),
                class: arg_of(user, "class").map(str::to_string),
                uid: arg_of(user, "uid").map(str::to_string),
                full_name: arg_of(user, "full-name").map(str::to_string),
                authentication_methods: child_block(user, "authentication")
                    .map(authentication_methods)
                    .unwrap_or_default(),
            })
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

    #[test]
    fn no_login_is_none() {
        assert!(system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n"))
            .unwrap()
            .login
            .is_none());
    }
}
