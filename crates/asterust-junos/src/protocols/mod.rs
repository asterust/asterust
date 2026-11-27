//! `protocols`
//! `protocols { ... }`.
//!
//! # See also
//!
//! - [`[edit protocols]` hierarchy](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/topic-map/hierarchy-edit-protocols.html)

pub mod bgp;
pub mod isis;
pub mod ospf;
mod traceoptions;

pub use bgp::{
    AddPath, AddPathSend, Bgp, BgpBfd, BgpFamily, BgpGracefulRestart, BgpGroup, BgpMultipath, BgpNeighbor,
    BgpPeerOptions, DynamicNeighbor, EntropyLabel, FamilyGracefulRestart, LocalAs, Multihop, OutputQueuePriority,
    PathSelection, PeerAutoDiscovery, PrefixLimit, RemovePrivate,
};
pub use isis::{
    Isis, IsisBfd, IsisGracefulRestart, IsisInterface, IsisInterfaceLevel, IsisLevel, IsisOverload, IsisSpfOptions,
};
pub use ospf::{
    Ospf, OspfArea, OspfAreaRange, OspfAuthentication, OspfBackupSpfOptions, OspfBfd, OspfDatabaseProtection,
    OspfGracefulRestart, OspfInterface, OspfInterfaceTopology, OspfMd5Key, OspfNeighbor, OspfNssa, OspfOverload,
    OspfPostConvergenceLfa, OspfSourcePacketRouting, OspfSpfOptions, OspfStub, OspfTopology, OspfTrafficEngineering,
    OspfVirtualLink,
};
pub use traceoptions::{TraceFlag, Traceoptions};
