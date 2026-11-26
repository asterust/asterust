"""Hierarchical network device config parsing. Platform views live in submodules, e.g. ``asterust.iosxr``."""

from asterust._core import Config, ConfigBlock, ConfigLine, parse_cisco, parse_juniper

__all__ = ["Config", "ConfigBlock", "ConfigLine", "parse_cisco", "parse_juniper"]
