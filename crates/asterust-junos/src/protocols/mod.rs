//! `protocols { ... }`: one module per protocol.

pub mod bgp;
pub mod isis;
pub mod ospf;

pub use bgp::{Bgp, BgpGroup, BgpNeighbor};
pub use isis::{Isis, IsisInterface};
pub use ospf::{Ospf, OspfArea, OspfInterface};
