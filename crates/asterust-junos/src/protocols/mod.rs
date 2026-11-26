//! Groups the supported `protocols` statements, one module per protocol.
//!
//! Configuration path: `protocols { ... }`.
//!
//! # See also
//!
//! - [`[edit protocols]` hierarchy](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/topic-map/hierarchy-edit-protocols.html)

pub mod bgp;
pub mod isis;
pub mod ospf;

pub use bgp::{Bgp, BgpGroup, BgpNeighbor};
pub use isis::{Isis, IsisInterface};
pub use ospf::{Ospf, OspfArea, OspfInterface};
