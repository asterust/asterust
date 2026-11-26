# asterust (Python)

Python bindings for [asterust](https://github.com/asterust/asterust): parse
network device configs into a hierarchical tree, with typed platform views.
Everything is compiled into one wheel; import only the submodule you need.

```python
from asterust.iosxr import IosXrDevice

device = IosXrDevice(running_config_text)

for iface in device.interfaces:
    print(iface.name, iface.ipv4_addresses, iface.shutdown)

device.ospf   # list of OspfProcess
device.bgp    # list of BgpProcess
device.isis   # list of IsisInstance
```

The generic tree is available for any platform:

```python
from asterust import parse_cisco, parse_juniper

config = parse_cisco(text)
for block in config.find_blocks_by_pattern(r"router ospf"):
    print(block.root_line.text)
```

## Development

```sh
make py-test  # from the repo root: build into a venv and run pytest
```
