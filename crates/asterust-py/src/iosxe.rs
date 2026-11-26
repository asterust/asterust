//! Python wrappers for `asterust-iosxe`. The data classes mirror the Rust structs field for field.
//!
//! Class names carry an `IosXe` prefix because they share the flat `_core` module with the IOS XR
//! ones; `asterust.iosxe` re-exports them under their short names.

use crate::config::PyConfig;
use asterust_iosxe::bgp::{BgpNeighbor, BgpProcess};
use asterust_iosxe::interface::Interface;
use asterust_iosxe::isis::IsisInstance;
use asterust_iosxe::ospf::{OspfNetwork, OspfProcess};
use asterust_iosxe::IosXeDevice;
use pyo3::prelude::*;

#[pyclass(name = "IosXeInterface", module = "asterust.iosxe", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyIosXeInterface {
    name: String,
    description: Option<String>,
    vrf: Option<String>,
    ipv4_addresses: Vec<String>,
    ipv6_addresses: Vec<String>,
    shutdown: bool,
    ospf_process_id: Option<String>,
    ospf_area: Option<String>,
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

#[pyclass(name = "IosXeIsisInstance", module = "asterust.iosxe", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyIosXeIsisInstance {
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

impl_repr!(
    PyIosXeInterface,
    PyIosXeOspfNetwork,
    PyIosXeOspfProcess,
    PyIosXeBgpNeighbor,
    PyIosXeBgpProcess,
    PyIosXeIsisInstance
);

/// A parsed IOS XE running config. `interfaces`, `ospf`, `bgp` and `isis` extract typed blocks.
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

    /// Build from an already-parsed `asterust.Config`.
    #[staticmethod]
    fn from_config(config: PyRef<'_, PyConfig>) -> Self {
        Self {
            inner: IosXeDevice::from_config(config.inner.clone()),
        }
    }

    #[getter]
    fn config(&self) -> PyConfig {
        PyConfig {
            inner: self.inner.config().clone(),
        }
    }

    #[getter]
    fn interfaces(&self) -> Vec<PyIosXeInterface> {
        self.inner.interfaces().into_iter().map(Into::into).collect()
    }

    #[getter]
    fn ospf(&self) -> Vec<PyIosXeOspfProcess> {
        self.inner.ospf().into_iter().map(Into::into).collect()
    }

    #[getter]
    fn bgp(&self) -> Vec<PyIosXeBgpProcess> {
        self.inner.bgp().into_iter().map(Into::into).collect()
    }

    #[getter]
    fn isis(&self) -> Vec<PyIosXeIsisInstance> {
        self.inner.isis().into_iter().map(Into::into).collect()
    }
}
