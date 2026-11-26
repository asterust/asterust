"""Typed Cisco IOS XE views: ``IosXeDevice(text).interfaces`` / ``.ospf`` / ``.bgp`` / ``.isis``."""

from asterust._core import IosXeBgpNeighbor as BgpNeighbor
from asterust._core import IosXeBgpProcess as BgpProcess
from asterust._core import IosXeDevice
from asterust._core import IosXeInterface as Interface
from asterust._core import IosXeIsisInstance as IsisInstance
from asterust._core import IosXeOspfNetwork as OspfNetwork
from asterust._core import IosXeOspfProcess as OspfProcess

__all__ = [
    "BgpNeighbor",
    "BgpProcess",
    "Interface",
    "IosXeDevice",
    "IsisInstance",
    "OspfNetwork",
    "OspfProcess",
]
