//! Python wrappers for `asterust-iosxr`. The data classes mirror the Rust structs field for field.

use crate::config::PyConfig;
use asterust_iosxr::bgp::{BgpNeighbor, BgpProcess};
use asterust_iosxr::interface::Interface;
use asterust_iosxr::isis::IsisInstance;
use asterust_iosxr::ospf::{OspfArea, OspfProcess};
use asterust_iosxr::IosXrDevice;
use pyo3::prelude::*;

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

#[pyclass(name = "BgpNeighbor", module = "asterust.iosxr", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyBgpNeighbor {
    address: String,
    remote_as: Option<String>,
    description: Option<String>,
}

impl From<BgpNeighbor> for PyBgpNeighbor {
    fn from(n: BgpNeighbor) -> Self {
        Self {
            address: n.address,
            remote_as: n.remote_as,
            description: n.description,
        }
    }
}

#[pyclass(name = "BgpProcess", module = "asterust.iosxr", frozen, get_all, eq)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyBgpProcess {
    asn: String,
    router_id: Option<String>,
    address_families: Vec<String>,
    neighbors: Vec<PyBgpNeighbor>,
}

impl From<BgpProcess> for PyBgpProcess {
    fn from(p: BgpProcess) -> Self {
        Self {
            asn: p.asn,
            router_id: p.router_id,
            address_families: p.address_families,
            neighbors: p.neighbors.into_iter().map(Into::into).collect(),
        }
    }
}

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

impl_repr!(
    PyInterface,
    PyOspfArea,
    PyOspfProcess,
    PyBgpNeighbor,
    PyBgpProcess,
    PyIsisInstance
);

/// A parsed IOS XR running config. `interfaces`, `ospf`, `bgp` and `isis` extract typed blocks.
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

    /// Build from an already-parsed `asterust.Config`.
    #[staticmethod]
    fn from_config(config: PyRef<'_, PyConfig>) -> Self {
        Self {
            inner: IosXrDevice::from_config(config.inner.clone()),
        }
    }

    #[getter]
    fn config(&self) -> PyConfig {
        PyConfig {
            inner: self.inner.config().clone(),
        }
    }

    #[getter]
    fn interfaces(&self) -> Vec<PyInterface> {
        self.inner.interfaces().into_iter().map(Into::into).collect()
    }

    #[getter]
    fn ospf(&self) -> Vec<PyOspfProcess> {
        self.inner.ospf().into_iter().map(Into::into).collect()
    }

    #[getter]
    fn bgp(&self) -> Vec<PyBgpProcess> {
        self.inner.bgp().into_iter().map(Into::into).collect()
    }

    #[getter]
    fn isis(&self) -> Vec<PyIsisInstance> {
        self.inner.isis().into_iter().map(Into::into).collect()
    }
}
