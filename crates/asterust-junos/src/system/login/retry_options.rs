//! `retry-options`
//! `system { login { retry-options { ... } } }`.
//!
//! # See also
//!
//! - [retry-options](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/retry-options-edit-system.html)

use crate::arg_of;
use asterust::config::ConfigLine;

/// The `system login retry-options` block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LoginRetryOptions {
    pub backoff_factor: Option<String>,
    pub backoff_threshold: Option<String>,
    /// Minutes.
    pub lockout_period: Option<String>,
    pub maximum_time: Option<String>,
    pub minimum_time: Option<String>,
    pub tries_before_disconnect: Option<String>,
}

pub(super) fn parse_retry_options(retry: &ConfigLine) -> LoginRetryOptions {
    let field = |keyword: &str| arg_of(retry, keyword).map(str::to_string);
    LoginRetryOptions {
        backoff_factor: field("backoff-factor"),
        backoff_threshold: field("backoff-threshold"),
        lockout_period: field("lockout-period"),
        maximum_time: field("maximum-time"),
        minimum_time: field("minimum-time"),
        tries_before_disconnect: field("tries-before-disconnect"),
    }
}
