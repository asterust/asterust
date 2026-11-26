# asterust

[![CI](https://github.com/asterust/asterust/actions/workflows/ci.yml/badge.svg)](https://github.com/asterust/asterust/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/asterust.svg)](https://crates.io/crates/asterust)
[![PyPI](https://img.shields.io/pypi/v/asterust.svg)](https://pypi.org/project/asterust/)
[![docs.rs](https://docs.rs/asterust/badge.svg)](https://docs.rs/asterust)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

Hierarchical parsing of network device configuration, in Rust and Python.

- **A generic config tree** (`Config` / `ConfigBlock` / `ConfigLine`) with a parser per syntax family:
  Cisco-style indentation and Juniper-style braces.
- **Typed platform views** on top of it: pull interfaces, OSPF, BGP, IS-IS and VRFs out of an IOS XR or
  IOS XE running config as plain structs instead of regexes.
- **Python bindings** that ship everything in one wheel.

The core crate is pure Rust with no PyO3 (or other binding) dependency, so any tool can depend on it
directly. Bindings and platform support live in separate crates in this workspace.

## Packages

| Package | What it is | Install |
| --- | --- | --- |
| `asterust` (root) | Config tree and the `cisco` / `juniper` parsers | `cargo add asterust` |
| [`asterust-iosxr`](crates/asterust-iosxr) | IOS XR views: interface, OSPF, BGP, IS-IS, VRF | `cargo add asterust-iosxr` |
| [`asterust-iosxe`](crates/asterust-iosxe) | IOS XE views: interface, OSPF, BGP, IS-IS, VRF | `cargo add asterust-iosxe` |
| [`asterust-py`](crates/asterust-py) | PyO3 bindings, published to PyPI as `asterust` | `pip install asterust` |

Rust users depend only on the platform they need. The Python wheel bundles all platforms; import only the
submodule you use.

## Usage

### Rust

```rust
use asterust_iosxr::IosXrDevice;

let device = IosXrDevice::parse(running_config_text);
for iface in device.interfaces() {
    println!("{}: {:?}", iface.name, iface.ipv4_addresses);
}
for bgp in device.bgp() {
    println!("AS {}: {} neighbors", bgp.asn, bgp.neighbors.len());
}
```

The generic tree works for any platform:

```rust
use asterust::config::cisco;

let config = cisco::parse(raw_running_config_text);
let ospf_blocks = config.find_blocks_by_pattern(r"router ospf");
```

### Python

```python
from asterust.iosxr import IosXrDevice

device = IosXrDevice(running_config_text)
for iface in device.interfaces:
    print(iface.name, iface.ipv4_addresses, iface.shutdown)

device.ospf  # list of OspfProcess
device.bgp   # list of BgpProcess
device.isis  # list of IsisInstance
```

`asterust.iosxe.IosXeDevice` works the same way, and `asterust.parse_cisco` / `parse_juniper` expose the
generic tree. See the [Python package README](crates/asterust-py/README.md).

## Supported syntax

- `config::cisco`: indentation-based hierarchy, `!`-terminated blocks (IOS, IOS-XE, IOS-XR, NX-OS and
  similar CLIs).
- `config::juniper`: brace-delimited hierarchy, `;`-terminated leaves (JunOS `show configuration`).

Typed views exist for IOS XR and IOS XE. Each platform crate's README lists what it extracts and which
Cisco documentation its test fixtures come from.

## Development

```sh
make test     # Rust unit + integration tests (whole workspace)
make lint     # clippy, warnings as errors
make fmt      # format with rustfmt
make check    # everything CI runs (fmt --check, lint, test)
make py-test  # build the Python extension into a venv and run pytest
```

## Releases

Use conventional commit prefixes (`feat:`, `fix:`, `feat!:`, `docs:`, ...). [release-plz](https://release-plz.dev)
opens a release PR that bumps each crate that changed and updates its changelog; merging it publishes to
crates.io (Trusted Publishing), tags `<package>-v<version>`, creates the GitHub Release, and uploads the
Python wheels to PyPI. Don't bump versions by hand.
