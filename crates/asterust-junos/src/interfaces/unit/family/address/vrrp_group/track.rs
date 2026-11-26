//! `... address <address> { vrrp-group <id> { track { ... } } }`.

use crate::{active_children, arg_of, args_of, named_children, value_after};
use asterust::config::ConfigLine;

/// A `bandwidth-threshold` of a tracked interface: the priority cost applied below `bandwidth` bits per second.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BandwidthThreshold {
    pub bandwidth: String,
    pub priority_cost: Option<String>,
}

/// A `track interface <name>`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TrackedInterface {
    pub name: String,
    pub priority_cost: Option<String>,
    pub bandwidth_thresholds: Vec<BandwidthThreshold>,
}

/// A `track route <prefix> routing-instance <name> priority-cost <n>` line.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TrackedRoute {
    pub prefix: String,
    pub routing_instance: Option<String>,
    pub priority_cost: Option<String>,
}

/// The `track` block of a VRRP group.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct VrrpTrack {
    pub interfaces: Vec<TrackedInterface>,
    pub routes: Vec<TrackedRoute>,
    pub priority_hold_time: Option<String>,
}

pub(super) fn parse_track(track: &ConfigLine) -> VrrpTrack {
    VrrpTrack {
        interfaces: named_children(track, "interface")
            .map(|(name, interface)| TrackedInterface {
                name: name.to_string(),
                priority_cost: arg_of(interface, "priority-cost").map(str::to_string),
                bandwidth_thresholds: active_children(interface)
                    .filter_map(|c| c.arg("bandwidth-threshold"))
                    .map(parse_bandwidth_threshold)
                    .collect(),
            })
            .collect(),
        routes: args_of(track, "route").into_iter().map(parse_route).collect(),
        priority_hold_time: arg_of(track, "priority-hold-time").map(str::to_string),
    }
}

/// `<bits-per-second> priority-cost <n>`.
fn parse_bandwidth_threshold(arg: &str) -> BandwidthThreshold {
    let words: Vec<&str> = arg.split_whitespace().collect();
    BandwidthThreshold {
        bandwidth: words.first().unwrap_or(&"").to_string(),
        priority_cost: value_after(&words, "priority-cost"),
    }
}

/// `<prefix> routing-instance <name> priority-cost <n>`.
fn parse_route(arg: &str) -> TrackedRoute {
    let words: Vec<&str> = arg.split_whitespace().collect();
    TrackedRoute {
        prefix: words.first().unwrap_or(&"").to_string(),
        routing_instance: value_after(&words, "routing-instance"),
        priority_cost: value_after(&words, "priority-cost"),
    }
}
