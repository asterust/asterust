//! Python wrappers for `asterust-iosxr`. The data classes mirror the Rust structs field for field.
//! BGP peer and address-family settings are flattened onto each class that carries them.

use crate::config::PyConfig;
use asterust_iosxr::bgp::{
    BgpAddressFamily, BgpAfGroup, BgpNeighbor, BgpNeighborAddressFamily, BgpNeighborGroup, BgpProcess, BgpSessionGroup,
    BgpTimers, BgpVrf,
};
use asterust_iosxr::interface::Interface;
use asterust_iosxr::isis::IsisInstance;
use asterust_iosxr::ospf::{OspfArea, OspfProcess};
use asterust_iosxr::vrf::{Vrf, VrfAddressFamily};
use asterust_iosxr::IosXrDevice;
use pyo3::prelude::*;

/// An IOS XR interface. Address fields hold the raw argument text
/// (e.g. `"10.0.0.1 255.255.255.0"`, `"2001:db8::1/64"`).
#[pyclass(name = "Interface", module = "asterust.iosxr", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyInterface {
    name: String,
    description: Option<String>,
    vrf: Option<String>,
    ipv4_addresses: Vec<String>,
    ipv6_addresses: Vec<String>,
    shutdown: bool,
}

impl From<Interface> for PyInterface {
    fn from(i: Interface) -> Self {
        Self {
            name: i.name,
            description: i.description,
            vrf: i.vrf,
            ipv4_addresses: i.ipv4_addresses,
            ipv6_addresses: i.ipv6_addresses,
            shutdown: i.shutdown,
        }
    }
}

/// An OSPF area and the interfaces configured under it.
#[pyclass(name = "OspfArea", module = "asterust.iosxr", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyOspfArea {
    id: String,
    interfaces: Vec<String>,
}

impl From<OspfArea> for PyOspfArea {
    fn from(a: OspfArea) -> Self {
        Self {
            id: a.id,
            interfaces: a.interfaces,
        }
    }
}

/// An IOS XR OSPF process.
#[pyclass(name = "OspfProcess", module = "asterust.iosxr", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyOspfProcess {
    process_id: String,
    router_id: Option<String>,
    areas: Vec<PyOspfArea>,
}

impl From<OspfProcess> for PyOspfProcess {
    fn from(p: OspfProcess) -> Self {
        Self {
            process_id: p.process_id,
            router_id: p.router_id,
            areas: p.areas.into_iter().map(Into::into).collect(),
        }
    }
}

/// Keepalive and hold-time from `timers [bgp] <keepalive> <hold-time>`.
#[pyclass(name = "BgpTimers", module = "asterust.iosxr", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyBgpTimers {
    keepalive: u32,
    hold_time: u32,
}

impl From<BgpTimers> for PyBgpTimers {
    fn from(t: BgpTimers) -> Self {
        Self {
            keepalive: t.keepalive,
            hold_time: t.hold_time,
        }
    }
}

/// Defines a Python class made of some identifying string fields plus the shared address-family
/// settings.
macro_rules! af_settings_class {
    ($rust:ident, $py:literal, $doc:literal, $src:ty, [$($key:ident),+]) => {
        #[doc = $doc]
        #[pyclass(name = $py, module = "asterust.iosxr", frozen, get_all, eq)]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $rust {
            $($key: String,)+
            route_policy_in: Option<String>,
            route_policy_out: Option<String>,
            route_reflector_client: bool,
            next_hop_self: bool,
            soft_reconfiguration_inbound: bool,
            send_community_ebgp: bool,
            send_extended_community_ebgp: bool,
            default_originate: bool,
            remove_private_as: bool,
            as_override: bool,
            allowas_in: Option<u32>,
            site_of_origin: Option<String>,
            maximum_prefix: Option<u32>,
            weight: Option<u32>,
            use_af_group: Option<String>,
        }

        impl From<$src> for $rust {
            fn from(src: $src) -> Self {
                let s = src.settings;
                Self {
                    $($key: src.$key,)+
                    route_policy_in: s.route_policy_in,
                    route_policy_out: s.route_policy_out,
                    route_reflector_client: s.route_reflector_client,
                    next_hop_self: s.next_hop_self,
                    soft_reconfiguration_inbound: s.soft_reconfiguration_inbound,
                    send_community_ebgp: s.send_community_ebgp,
                    send_extended_community_ebgp: s.send_extended_community_ebgp,
                    default_originate: s.default_originate,
                    remove_private_as: s.remove_private_as,
                    as_override: s.as_override,
                    allowas_in: s.allowas_in,
                    site_of_origin: s.site_of_origin,
                    maximum_prefix: s.maximum_prefix,
                    weight: s.weight,
                    use_af_group: s.use_af_group,
                }
            }
        }
    };
}

af_settings_class!(
    PyBgpNeighborAddressFamily,
    "BgpNeighborAddressFamily",
    "An `address-family <afi> <safi>` section under a neighbor or neighbor group.",
    BgpNeighborAddressFamily,
    [name]
);
af_settings_class!(
    PyBgpAfGroup,
    "BgpAfGroup",
    "An `af-group <name> address-family <afi> <safi>` template.",
    BgpAfGroup,
    [name, address_family]
);

/// Defines a Python class made of one identifying string field plus the shared peer settings.
macro_rules! peer_settings_class {
    ($rust:ident, $py:literal, $doc:literal, $src:ty, $key:ident) => {
        #[doc = $doc]
        #[pyclass(name = $py, module = "asterust.iosxr", frozen, get_all, eq)]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $rust {
            $key: String,
            remote_as: Option<String>,
            description: Option<String>,
            update_source: Option<String>,
            ebgp_multihop: Option<u32>,
            advertisement_interval: Option<u32>,
            timers: Option<PyBgpTimers>,
            shutdown: bool,
            use_neighbor_group: Option<String>,
            use_session_group: Option<String>,
            address_families: Vec<PyBgpNeighborAddressFamily>,
        }

        impl From<$src> for $rust {
            fn from(src: $src) -> Self {
                let s = src.settings;
                Self {
                    $key: src.$key,
                    remote_as: s.remote_as,
                    description: s.description,
                    update_source: s.update_source,
                    ebgp_multihop: s.ebgp_multihop,
                    advertisement_interval: s.advertisement_interval,
                    timers: s.timers.map(Into::into),
                    shutdown: s.shutdown,
                    use_neighbor_group: s.use_neighbor_group,
                    use_session_group: s.use_session_group,
                    address_families: s.address_families.into_iter().map(Into::into).collect(),
                }
            }
        }
    };
}

peer_settings_class!(
    PyBgpNeighbor,
    "BgpNeighbor",
    "A `neighbor <address>` block. Peer settings are reachable directly (`neighbor.remote_as`).",
    BgpNeighbor,
    address
);
peer_settings_class!(
    PyBgpNeighborGroup,
    "BgpNeighborGroup",
    "A `neighbor-group <name>` template.",
    BgpNeighborGroup,
    name
);
peer_settings_class!(
    PyBgpSessionGroup,
    "BgpSessionGroup",
    "A `session-group <name>` template (address-family-independent settings only).",
    BgpSessionGroup,
    name
);

/// A router- or VRF-level `address-family <afi> <safi>` section. Values are the raw argument text
/// (e.g. `"10.2.0.0/24 route-policy foo"`).
#[pyclass(name = "BgpAddressFamily", module = "asterust.iosxr", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyBgpAddressFamily {
    /// e.g. `"ipv4 unicast"`.
    name: String,
    networks: Vec<String>,
    aggregate_addresses: Vec<String>,
    redistributions: Vec<String>,
}

impl From<BgpAddressFamily> for PyBgpAddressFamily {
    fn from(a: BgpAddressFamily) -> Self {
        Self {
            name: a.name,
            networks: a.networks,
            aggregate_addresses: a.aggregate_addresses,
            redistributions: a.redistributions,
        }
    }
}

/// A `vrf <name>` section under `router bgp`.
#[pyclass(name = "BgpVrf", module = "asterust.iosxr", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyBgpVrf {
    name: String,
    rd: Option<String>,
    router_id: Option<String>,
    address_families: Vec<PyBgpAddressFamily>,
    neighbors: Vec<PyBgpNeighbor>,
}

impl From<BgpVrf> for PyBgpVrf {
    fn from(v: BgpVrf) -> Self {
        Self {
            name: v.name,
            rd: v.rd,
            router_id: v.router_id,
            address_families: v.address_families.into_iter().map(Into::into).collect(),
            neighbors: v.neighbors.into_iter().map(Into::into).collect(),
        }
    }
}

/// An IOS XR BGP process. `asn` is kept as text since it may be asdot (`"1.10"`).
#[pyclass(name = "BgpProcess", module = "asterust.iosxr", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyBgpProcess {
    asn: String,
    router_id: Option<String>,
    /// `timers bgp <keepalive> <hold-time>` (default for all neighbors).
    timers: Option<PyBgpTimers>,
    default_local_preference: Option<u32>,
    cluster_id: Option<String>,
    confederation_identifier: Option<String>,
    confederation_peers: Vec<String>,
    nsr: bool,
    address_families: Vec<PyBgpAddressFamily>,
    neighbors: Vec<PyBgpNeighbor>,
    neighbor_groups: Vec<PyBgpNeighborGroup>,
    session_groups: Vec<PyBgpSessionGroup>,
    af_groups: Vec<PyBgpAfGroup>,
    vrfs: Vec<PyBgpVrf>,
}

impl From<BgpProcess> for PyBgpProcess {
    fn from(p: BgpProcess) -> Self {
        Self {
            asn: p.asn,
            router_id: p.router_id,
            timers: p.timers.map(Into::into),
            default_local_preference: p.default_local_preference,
            cluster_id: p.cluster_id,
            confederation_identifier: p.confederation_identifier,
            confederation_peers: p.confederation_peers,
            nsr: p.nsr,
            address_families: p.address_families.into_iter().map(Into::into).collect(),
            neighbors: p.neighbors.into_iter().map(Into::into).collect(),
            neighbor_groups: p.neighbor_groups.into_iter().map(Into::into).collect(),
            session_groups: p.session_groups.into_iter().map(Into::into).collect(),
            af_groups: p.af_groups.into_iter().map(Into::into).collect(),
            vrfs: p.vrfs.into_iter().map(Into::into).collect(),
        }
    }
}

/// An IOS XR IS-IS instance.
#[pyclass(name = "IsisInstance", module = "asterust.iosxr", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyIsisInstance {
    tag: String,
    nets: Vec<String>,
    interfaces: Vec<String>,
}

impl From<IsisInstance> for PyIsisInstance {
    fn from(i: IsisInstance) -> Self {
        Self {
            tag: i.tag,
            nets: i.nets,
            interfaces: i.interfaces,
        }
    }
}

/// An `address-family <afi> <safi>` section of a VRF definition.
#[pyclass(name = "VrfAddressFamily", module = "asterust.iosxr", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyVrfAddressFamily {
    /// e.g. `"ipv4 unicast"`.
    name: String,
    maximum_prefix: Option<u32>,
    import_route_policy: Option<String>,
    export_route_policy: Option<String>,
    import_route_targets: Vec<String>,
    export_route_targets: Vec<String>,
}

impl From<VrfAddressFamily> for PyVrfAddressFamily {
    fn from(a: VrfAddressFamily) -> Self {
        Self {
            name: a.name,
            maximum_prefix: a.maximum_prefix,
            import_route_policy: a.import_route_policy,
            export_route_policy: a.export_route_policy,
            import_route_targets: a.import_route_targets,
            export_route_targets: a.export_route_targets,
        }
    }
}

/// An IOS XR VRF definition.
#[pyclass(name = "Vrf", module = "asterust.iosxr", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyVrf {
    name: String,
    description: Option<String>,
    address_families: Vec<PyVrfAddressFamily>,
}

impl From<Vrf> for PyVrf {
    fn from(v: Vrf) -> Self {
        Self {
            name: v.name,
            description: v.description,
            address_families: v.address_families.into_iter().map(Into::into).collect(),
        }
    }
}

impl_repr!(
    PyInterface,
    PyOspfArea,
    PyOspfProcess,
    PyBgpTimers,
    PyBgpNeighborAddressFamily,
    PyBgpAfGroup,
    PyBgpNeighbor,
    PyBgpNeighborGroup,
    PyBgpSessionGroup,
    PyBgpAddressFamily,
    PyBgpVrf,
    PyBgpProcess,
    PyIsisInstance,
    PyVrfAddressFamily,
    PyVrf
);

/// Holds a parsed IOS XR running config. The `interfaces`, `ospf`, `bgp`, `isis` and `vrfs`
/// getters extract typed blocks.
///
/// ```python
/// from asterust.iosxr import IosXrDevice
///
/// device = IosXrDevice(running_config_text)
/// for iface in device.interfaces:
///     print(iface.name, iface.ipv4_addresses, iface.shutdown)
/// ```
#[pyclass(name = "IosXrDevice", module = "asterust.iosxr", frozen)]
pub struct PyIosXrDevice {
    inner: IosXrDevice,
}

#[pymethods]
impl PyIosXrDevice {
    #[new]
    fn new(config_text: &str) -> Self {
        Self {
            inner: IosXrDevice::parse(config_text),
        }
    }

    /// Builds a device from an already-parsed `asterust.Config`.
    #[staticmethod]
    fn from_config(config: PyRef<'_, PyConfig>) -> Self {
        Self {
            inner: IosXrDevice::from_config(config.inner.clone()),
        }
    }

    /// Returns the underlying generic `Config` tree.
    #[getter]
    fn config(&self) -> PyConfig {
        PyConfig {
            inner: self.inner.config().clone(),
        }
    }

    /// Returns every `interface` block as an `Interface`.
    #[getter]
    fn interfaces(&self) -> Vec<PyInterface> {
        self.inner.interfaces().into_iter().map(Into::into).collect()
    }

    /// Returns every `router ospf` process as an `OspfProcess`.
    #[getter]
    fn ospf(&self) -> Vec<PyOspfProcess> {
        self.inner.ospf().into_iter().map(Into::into).collect()
    }

    /// Returns every `router bgp` process as a `BgpProcess`.
    #[getter]
    fn bgp(&self) -> Vec<PyBgpProcess> {
        self.inner.bgp().into_iter().map(Into::into).collect()
    }

    /// Returns every `router isis` instance as an `IsisInstance`.
    #[getter]
    fn isis(&self) -> Vec<PyIsisInstance> {
        self.inner.isis().into_iter().map(Into::into).collect()
    }

    /// Returns every `vrf` definition as a `Vrf`.
    #[getter]
    fn vrfs(&self) -> Vec<PyVrf> {
        self.inner.vrfs().into_iter().map(Into::into).collect()
    }
}
