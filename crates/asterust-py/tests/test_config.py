from asterust import parse_cisco, parse_juniper

CISCO = """
interface Loopback0
 ipv4 address 10.255.0.1 255.255.255.255
!
router ospf 1
 router-id 10.255.0.1
!
"""

JUNIPER = """
interfaces {
    ge-0/0/0 {
        description uplink;
    }
}
"""


def test_cisco_tree_navigation():
    config = parse_cisco(CISCO)
    assert len(config) == 2
    block = config.find_blocks_by_pattern(r"router ospf")[0]
    assert block.root_line.text == "router ospf 1"
    assert block.find_child_with("router-id").match_str_group(r"router-id (\S+)") == "10.255.0.1"
    assert block.find_child_with("nope") is None


def test_juniper_tree():
    config = parse_juniper(JUNIPER)
    assert len(config) == 1
    assert config.blocks[0].root_line.text == "interfaces"
    assert config.blocks[0].find_line_with("description") is not None
