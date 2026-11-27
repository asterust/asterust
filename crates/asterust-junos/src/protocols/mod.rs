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
    AddPath, AddPathSend, Bgp, BgpBfd, BgpFamily, BgpGracefulRestart, BgpGroup, BgpMultipath, BgpNeighbor,
    BgpPeerOptions, DynamicNeighbor, EntropyLabel, FamilyGracefulRestart, LocalAs, Multihop, OutputQueuePriority,
    PathSelection, PeerAutoDiscovery, PrefixLimit, RemovePrivate,
};
pub use isis::{Isis, IsisInterface};
pub use ospf::{
    Ospf, OspfArea, OspfAuthentication, OspfBfd, OspfInterface, OspfInterfaceTopology, OspfMd5Key, OspfNeighbor,
};
