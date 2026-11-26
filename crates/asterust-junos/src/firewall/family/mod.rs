//! `firewall { family <family> { ... } }`: the filters of each address family.

mod filter;

pub use filter::{FilterAction, FilterActions, FilterTerm, FirewallFilter, MatchCondition};

use crate::named_children;
use asterust::config::ConfigLine;
use filter::parse_filter;

pub(super) fn filters(firewall: &ConfigLine) -> Vec<FirewallFilter> {
    named_children(firewall, "family")
        .flat_map(|(family, family_line)| {
            named_children(family_line, "filter").map(move |(name, filter)| parse_filter(family, name, filter))
        })
        .collect()
}
