//! `interfaces { <name> { unit <n> { vlan-tags ...; } } }`.

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The `vlan-tags` of a dual-tagged unit. Each tag is written `<tpid>.<vlan-id>`, e.g. `"0x8100.100"`, or as a bare
/// VLAN ID. `inner-list` is not extracted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct VlanTags {
    pub outer: Option<String>,
    pub inner: Option<String>,
    /// From `inner-range <tpid>.<vlan-id>-<vlan-id>`.
    pub inner_range: Option<String>,
}

pub(super) fn parse_vlan_tags(unit: &ConfigLine) -> Option<VlanTags> {
    let words = statement_words(unit, "vlan-tags")?;
    Some(VlanTags {
        outer: value_after(&words, "outer"),
        inner: value_after(&words, "inner"),
        inner_range: value_after(&words, "inner-range"),
    })
}
