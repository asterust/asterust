//! Typed views of common Cisco IOS XE config blocks, extracted from a tree
//! parsed by [`asterust::config::cisco`].
//!
//! Each module exposes a plain data struct with a `from_block` constructor
//! (returns `None` if the block isn't that kind) and a free function that
//! collects every matching block in a [`asterust::config::Config`].
//! [`IosXeDevice`] bundles a parsed config with all of them as getters.
//!
//! Unlike IOS XR, IOS XE puts several settings on flat lines: BGP neighbors
//! are `neighbor <addr> ...` lines under `router bgp`, and OSPF/IS-IS can be
//! enabled per interface (`ip ospf <id> area <area>`, `ip router isis`).

pub mod bgp;
pub mod device;
pub mod interface;
pub mod isis;
pub mod ospf;
pub mod vrf;

pub use device::IosXeDevice;

use asterust::config::ConfigLine;

/// Argument text following `keyword` in `text` (`"router-id 1.1.1.1"` with
/// keyword `"router-id"` gives `"1.1.1.1"`), or `None` if `text` doesn't
/// start with the whole keyword followed by a space.
pub(crate) fn line_arg<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    text.strip_prefix(keyword)?.strip_prefix(' ').map(str::trim)
}

/// [`line_arg`] applied to the first direct child of `parent` that matches.
pub(crate) fn child_arg<'a>(parent: &'a ConfigLine, keyword: &str) -> Option<&'a str> {
    parent.children.iter().find_map(|c| line_arg(&c.text, keyword))
}

/// [`line_arg`] applied to every direct child of `parent`, keeping matches.
pub(crate) fn child_args<'a>(parent: &'a ConfigLine, keyword: &str) -> Vec<&'a str> {
    parent
        .children
        .iter()
        .filter_map(|c| line_arg(&c.text, keyword))
        .collect()
}
