//! `protocols`
//! `protocols { ... }`.
//!
//! # See also
//!
//! - [`[edit protocols]` hierarchy](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/topic-map/hierarchy-edit-protocols.html)

pub mod bgp;
pub mod isis;
pub mod mpls;
pub mod ospf;
pub mod source_packet_routing;
mod traceoptions;

pub use bgp::{
    AddPath, AddPathSend, Bgp, BgpBfd, BgpFamily, BgpGracefulRestart, BgpGroup, BgpMultipath, BgpNeighbor,
    BgpPeerOptions, DynamicNeighbor, EntropyLabel, FamilyGracefulRestart, LocalAs, Multihop, OutputQueuePriority,
    PathSelection, PeerAutoDiscovery, PrefixLimit, RemovePrivate,
};
pub use bgp::{BgpSrv6, BgpSrv6Locator, BgpSrv6MicroSid};
pub use isis::{
    Isis, IsisBackupSpfOptions, IsisBfd, IsisDynamicOverload, IsisGracefulRestart, IsisInterface, IsisInterfaceLevel,
    IsisLevel, IsisOverload, IsisSourcePacketRouting, IsisSpfOptions, IsisTeFamily, IsisTopologies,
    IsisTrafficEngineering,
};
pub use mpls::{Mpls, MplsAdminGroup, MplsInterface, MplsLogUpdown, MplsPathMtu};
pub use ospf::{
    Ospf, OspfArea, OspfAreaRange, OspfAuthentication, OspfBackupSpfOptions, OspfBfd, OspfDatabaseProtection,
    OspfGracefulRestart, OspfInterface, OspfInterfaceTopology, OspfMd5Key, OspfNeighbor, OspfNssa, OspfOverload,
    OspfPostConvergenceLfa, OspfSourcePacketRouting, OspfSpfOptions, OspfStub, OspfTopology, OspfTrafficEngineering,
    OspfVirtualLink,
};
pub use source_packet_routing::{PathSegmentList, SegmentHop, SegmentList, SourcePacketRouting, SourceRoutingPath};
pub use traceoptions::{TraceFlag, Traceoptions};
