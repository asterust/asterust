//! `protocols`
//! `protocols { ... }`.
//!
//! # See also
//!
//! - [`[edit protocols]` hierarchy](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/topic-map/hierarchy-edit-protocols.html)

pub mod bgp;
pub mod isis;
pub mod ospf;

pub use bgp::{
    Bgp, BgpBfd, BgpGracefulRestart, BgpGroup, BgpMultipath, BgpNeighbor, BgpPeerOptions, DynamicNeighbor, LocalAs,
    Multihop, PathSelection, RemovePrivate,
};
pub use isis::{Isis, IsisInterface};
pub use ospf::{Ospf, OspfArea, OspfInterface};
