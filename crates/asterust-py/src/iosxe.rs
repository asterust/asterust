//! Python wrappers for `asterust-iosxe`. The data classes mirror the Rust structs field for field.
//!
//! Class names carry an `IosXe` prefix because they share the flat `_core` module with the IOS XR
//! ones; `asterust.iosxe` re-exports them under their short names.

use crate::config::PyConfig;
use asterust_iosxe::bgp::{BgpNeighbor, BgpProcess};
use asterust_iosxe::interface::Interface;
use asterust_iosxe::isis::IsisInstance;
use asterust_iosxe::ospf::{OspfNetwork, OspfProcess};
use asterust_iosxe::vrf::{Vrf, VrfAddressFamily};
use asterust_iosxe::IosXeDevice;
use pyo3::prelude::*;

/// An IOS XE interface. Address fields hold the raw argument text
/// (e.g. `"10.0.0.1 255.255.255.0"`, `"2001:db8::1/64"`).
#[pyclass(name = "IosXeInterface", module = "asterust.iosxe", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyIosXeInterface {
    name: String,
    description: Option<String>,
    vrf: Option<String>,
    ipv4_addresses: Vec<String>,
    ipv6_addresses: Vec<String>,
    shutdown: bool,
    /// Process id and area from `ip ospf <process> area <area>`.
    ospf_process_id: Option<String>,
    ospf_area: Option<String>,
    /// Tag from `ip router isis [tag]`; empty string for the untagged process.
    isis_tag: Option<String>,
}

impl From<Interface> for PyIosXeInterface {
    fn from(i: Interface) -> Self {
        Self {
            name: i.name,
            description: i.description,
            vrf: i.vrf,
            ipv4_addresses: i.ipv4_addresses,
            ipv6_addresses: i.ipv6_addresses,
            shutdown: i.shutdown,
            ospf_process_id: i.ospf_process_id,
            ospf_area: i.ospf_area,
            isis_tag: i.isis_tag,
        }
    }
}

/// A `network <prefix> <wildcard> area <area>` statement.
#[pyclass(name = "IosXeOspfNetwork", module = "asterust.iosxe", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyIosXeOspfNetwork {
    prefix: String,
    wildcard: String,
    area: String,
}

impl From<OspfNetwork> for PyIosXeOspfNetwork {
    fn from(n: OspfNetwork) -> Self {
        Self {
            prefix: n.prefix,
            wildcard: n.wildcard,
            area: n.area,
        }
    }
}

/// An IOS XE OSPF process. Interfaces enabled with `ip ospf <process> area <area>`
/// are on `IosXeInterface`, not here.
#[pyclass(name = "IosXeOspfProcess", module = "asterust.iosxe", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyIosXeOspfProcess {
    process_id: String,
    vrf: Option<String>,
    router_id: Option<String>,
    networks: Vec<PyIosXeOspfNetwork>,
    passive_interfaces: Vec<String>,
}

impl From<OspfProcess> for PyIosXeOspfProcess {
    fn from(p: OspfProcess) -> Self {
        Self {
            process_id: p.process_id,
            vrf: p.vrf,
            router_id: p.router_id,
            networks: p.networks.into_iter().map(Into::into).collect(),
            passive_interfaces: p.passive_interfaces,
        }
    }
}

/// A BGP neighbor assembled from its `neighbor <address> ...` lines.
#[pyclass(name = "IosXeBgpNeighbor", module = "asterust.iosxe", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyIosXeBgpNeighbor {
    address: String,
    remote_as: Option<String>,
    description: Option<String>,
    peer_group: Option<String>,
}

impl From<BgpNeighbor> for PyIosXeBgpNeighbor {
    fn from(n: BgpNeighbor) -> Self {
        Self {
            address: n.address,
            remote_as: n.remote_as,
            description: n.description,
            peer_group: n.peer_group,
        }
    }
}

/// An IOS XE BGP process. `asn` is kept as text since it may be asdot (`"1.10"`).
/// Peer-group definitions are not listed as neighbors.
#[pyclass(name = "IosXeBgpProcess", module = "asterust.iosxe", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyIosXeBgpProcess {
    asn: String,
    router_id: Option<String>,
    address_families: Vec<String>,
    neighbors: Vec<PyIosXeBgpNeighbor>,
}

impl From<BgpProcess> for PyIosXeBgpProcess {
    fn from(p: BgpProcess) -> Self {
        Self {
            asn: p.asn,
            router_id: p.router_id,
            address_families: p.address_families,
            neighbors: p.neighbors.into_iter().map(Into::into).collect(),
        }
    }
}

/// An IOS XE IS-IS process. Interfaces enabled with `ip router isis` are on
/// `IosXeInterface`, not here.
#[pyclass(name = "IosXeIsisInstance", module = "asterust.iosxe", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyIosXeIsisInstance {
    /// `None` for the untagged process (plain `router isis`).
    tag: Option<String>,
    nets: Vec<String>,
}

impl From<IsisInstance> for PyIosXeIsisInstance {
    fn from(i: IsisInstance) -> Self {
        Self {
            tag: i.tag,
            nets: i.nets,
        }
    }
}

/// Route targets and maps for one address family of a VRF.
#[pyclass(name = "IosXeVrfAddressFamily", module = "asterust.iosxe", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyIosXeVrfAddressFamily {
    /// e.g. `"ipv4"` or `"ipv6"`.
    name: String,
    /// `route-target both` entries appear in both lists.
    import_route_targets: Vec<String>,
    export_route_targets: Vec<String>,
    import_map: Option<String>,
    export_map: Option<String>,
}

impl From<VrfAddressFamily> for PyIosXeVrfAddressFamily {
    fn from(a: VrfAddressFamily) -> Self {
        Self {
            name: a.name,
            import_route_targets: a.import_route_targets,
            export_route_targets: a.export_route_targets,
            import_map: a.import_map,
            export_map: a.export_map,
        }
    }
}

/// An IOS XE VRF. The legacy `ip vrf` form has no address-family sections; its route targets and
/// maps are reported as a single `"ipv4"` address family.
#[pyclass(name = "IosXeVrf", module = "asterust.iosxe", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyIosXeVrf {
    name: String,
    description: Option<String>,
    rd: Option<String>,
    address_families: Vec<PyIosXeVrfAddressFamily>,
    /// `true` for `ip vrf <name>`, `false` for `vrf definition <name>`.
    legacy: bool,
}

impl From<Vrf> for PyIosXeVrf {
    fn from(v: Vrf) -> Self {
        Self {
            name: v.name,
            description: v.description,
            rd: v.rd,
            address_families: v.address_families.into_iter().map(Into::into).collect(),
            legacy: v.legacy,
        }
    }
}

impl_repr!(
    PyIosXeInterface,
    PyIosXeOspfNetwork,
    PyIosXeOspfProcess,
    PyIosXeBgpNeighbor,
    PyIosXeBgpProcess,
    PyIosXeIsisInstance,
    PyIosXeVrfAddressFamily,
    PyIosXeVrf
);

/// Holds a parsed IOS XE running config. The `interfaces`, `ospf`, `bgp`, `isis` and `vrfs`
/// getters extract typed blocks.
///
/// ```python
/// from asterust.iosxe import IosXeDevice
///
/// device = IosXeDevice(running_config_text)
/// for iface in device.interfaces:
///     print(iface.name, iface.ipv4_addresses, iface.shutdown)
/// ```
#[pyclass(name = "IosXeDevice", module = "asterust.iosxe", frozen)]
pub struct PyIosXeDevice {
    inner: IosXeDevice,
}

#[pymethods]
impl PyIosXeDevice {
    #[new]
    fn new(config_text: &str) -> Self {
        Self {
            inner: IosXeDevice::parse(config_text),
        }
    }

    /// Builds a device from an already-parsed `asterust.Config`.
    #[staticmethod]
    fn from_config(config: PyRef<'_, PyConfig>) -> Self {
        Self {
            inner: IosXeDevice::from_config(config.inner.clone()),
        }
    }

    /// Returns the underlying generic `Config` tree.
    #[getter]
    fn config(&self) -> PyConfig {
        PyConfig {
            inner: self.inner.config().clone(),
        }
    }

    /// Returns every `interface` block as an `IosXeInterface`.
    #[getter]
    fn interfaces(&self) -> Vec<PyIosXeInterface> {
        self.inner.interfaces().into_iter().map(Into::into).collect()
    }

    /// Returns every `router ospf` process as an `IosXeOspfProcess`.
    #[getter]
    fn ospf(&self) -> Vec<PyIosXeOspfProcess> {
        self.inner.ospf().into_iter().map(Into::into).collect()
    }

    /// Returns every `router bgp` process as an `IosXeBgpProcess`.
    #[getter]
    fn bgp(&self) -> Vec<PyIosXeBgpProcess> {
        self.inner.bgp().into_iter().map(Into::into).collect()
    }

    /// Returns every `router isis` process as an `IosXeIsisInstance`.
    #[getter]
    fn isis(&self) -> Vec<PyIosXeIsisInstance> {
        self.inner.isis().into_iter().map(Into::into).collect()
    }

    /// Returns every `vrf definition` (and legacy `ip vrf`) as an `IosXeVrf`.
    #[getter]
    fn vrfs(&self) -> Vec<PyIosXeVrf> {
        self.inner.vrfs().into_iter().map(Into::into).collect()
    }
}
