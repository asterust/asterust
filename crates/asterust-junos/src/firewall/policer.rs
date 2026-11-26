//! `policer`
//! `firewall { policer <name> { ... } }`.
//!
//! # See also
//!
//! - [policer (Configuring)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/policer-edit-firewall.html)

use crate::{arg_of, child_block, split_keyword, statements, unquote};
use asterust::config::ConfigLine;

/// A `policer <name>`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Policer {
    pub name: String,
    pub bandwidth_limit: Option<String>,
    pub bandwidth_percent: Option<String>,
    pub burst_size_limit: Option<String>,
    /// `then discard`.
    pub discard: bool,
    pub forwarding_class: Option<String>,
    pub loss_priority: Option<String>,
}

pub(super) fn parse_policer(name: &str, line: &ConfigLine) -> Policer {
    let limit = |keyword: &str| {
        child_block(line, "if-exceeding")
            .and_then(|block| arg_of(block, keyword))
            .map(str::to_string)
    };
    let mut policer = Policer {
        name: name.to_string(),
        bandwidth_limit: limit("bandwidth-limit"),
        bandwidth_percent: limit("bandwidth-percent"),
        burst_size_limit: limit("burst-size-limit"),
        discard: false,
        forwarding_class: None,
        loss_priority: None,
    };
    for statement in statements(line, "then") {
        let (action, arg) = split_keyword(statement);
        match action {
            "discard" => policer.discard = true,
            "forwarding-class" => policer.forwarding_class = Some(unquote(arg).to_string()),
            "loss-priority" => policer.loss_priority = Some(unquote(arg).to_string()),
            _ => {}
        }
    }
    policer
}
