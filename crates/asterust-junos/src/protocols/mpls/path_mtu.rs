//! `path-mtu`
//! `protocols { mpls { path-mtu { allow-fragmentation; rsvp { mtu-signaling; } } } }`.
//!
//! # See also
//!
//! - [path-mtu](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/path-mtu-edit-protocols-mpls.html)

use crate::{child_block, has_leaf};
use asterust::config::ConfigLine;

/// The `path-mtu` options of MPLS.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct MplsPathMtu {
    pub allow_fragmentation: bool,
    /// `rsvp { mtu-signaling; }`.
    pub rsvp_mtu_signaling: bool,
}

pub(super) fn parse_path_mtu(mpls: &ConfigLine) -> Option<MplsPathMtu> {
    let mtu = child_block(mpls, "path-mtu")?;
    Some(MplsPathMtu {
        allow_fragmentation: has_leaf(mtu, "allow-fragmentation"),
        rsvp_mtu_signaling: child_block(mtu, "rsvp").is_some_and(|rsvp| has_leaf(rsvp, "mtu-signaling")),
    })
}
