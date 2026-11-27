//! `output-queue-priority`
//! `protocols { bgp { output-queue-priority { ... } } }`.
//!
//! # See also
//!
//! - [BGP Route Prioritization](https://networkingdocs.hpe.com/documentation/us/en/software/junos/bgp/topics/topic-map/bgp-route-prioritization.html)

use crate::{active_children, child_block, value_after};
use asterust::config::ConfigLine;

/// The BGP output scheduler: the work tokens of each output queue and the queues behind the `low`, `medium` and
/// `high` keywords. The per-family `output-queue-priority` is on [`BgpFamily`](crate::protocols::bgp::BgpFamily).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct OutputQueuePriority {
    /// `update-tokens` of the `expedited` queue.
    pub expedited_tokens: Option<String>,
    /// The queue number and `update-tokens` of each `priority <n>` queue.
    pub priority_tokens: Vec<(String, String)>,
    /// A queue as written, e.g. `priority 10` or `expedited`.
    pub default_low: Option<String>,
    pub default_medium: Option<String>,
    pub default_high: Option<String>,
}

pub(super) fn parse_output_queue_priority(line: &ConfigLine) -> Option<OutputQueuePriority> {
    let scheduler = child_block(line, "output-queue-priority")?;
    let mut result = OutputQueuePriority::default();
    for child in active_children(scheduler) {
        let words: Vec<&str> = child
            .text
            .split_whitespace()
            .chain(active_children(child).flat_map(|grandchild| grandchild.text.split_whitespace()))
            .collect();
        match words.first().copied() {
            Some("expedited") => result.expedited_tokens = value_after(&words, "update-tokens"),
            Some("priority") => {
                if let (Some(queue), Some(tokens)) = (words.get(1), value_after(&words, "update-tokens")) {
                    result.priority_tokens.push((queue.to_string(), tokens));
                }
            }
            Some("defaults") => {
                result.default_low = result.default_low.or_else(|| default_queue(&words, "low"));
                result.default_medium = result.default_medium.or_else(|| default_queue(&words, "medium"));
                result.default_high = result.default_high.or_else(|| default_queue(&words, "high"));
            }
            _ => {}
        }
    }
    Some(result)
}

/// The words after `class` up to the next class, e.g. `priority 10`.
fn default_queue(words: &[&str], class: &str) -> Option<String> {
    const CLASSES: [&str; 3] = ["low", "medium", "high"];
    let start = words.iter().position(|word| *word == class)? + 1;
    let end = words[start..]
        .iter()
        .position(|word| CLASSES.contains(word))
        .map_or(words.len(), |offset| start + offset);
    Some(words[start..end].join(" "))
}

#[cfg(test)]
mod tests {
    use crate::protocols::bgp::bgp;
    use asterust::config::juniper;

    // Not from documentation: written from the BGP Route Prioritization guide. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    bgp {
        output-queue-priority {
            expedited update-tokens 100;
            priority 1 update-tokens 1;
            priority 2 {
                update-tokens 10;
            }
            defaults {
                low priority 1;
                medium priority 10;
                high expedited;
            }
        }
    }
}
"#;

    #[test]
    fn reads_the_tokens_of_each_queue() {
        let scheduler = bgp(&juniper::parse_braces(CONFIG))
            .unwrap()
            .output_queue_priority
            .unwrap();
        assert_eq!(scheduler.expedited_tokens.as_deref(), Some("100"));
        assert_eq!(
            scheduler.priority_tokens,
            [("1".to_string(), "1".to_string()), ("2".to_string(), "10".to_string())]
        );
    }

    #[test]
    fn reads_the_default_queues() {
        let scheduler = bgp(&juniper::parse_braces(CONFIG))
            .unwrap()
            .output_queue_priority
            .unwrap();
        assert_eq!(scheduler.default_low.as_deref(), Some("priority 1"));
        assert_eq!(scheduler.default_medium.as_deref(), Some("priority 10"));
        assert_eq!(scheduler.default_high.as_deref(), Some("expedited"));
    }

    // Not from documentation: written from the CLI Reference and the BGP Route Prioritization guide.
    const FAMILY: &str = r#"
protocols {
    bgp {
        group SR {
            family inet {
                unicast {
                    extended-nexthop-color;
                    extended-nexthop-tunnel;
                    output-queue-priority priority 7;
                    route-refresh-priority priority 8;
                    withdraw-priority expedited;
                }
                labeled-unicast;
            }
        }
    }
}
"#;

    #[test]
    fn reads_the_per_family_priorities_and_extended_nexthop_options() {
        let bgp = bgp(&juniper::parse_braces(FAMILY)).unwrap();
        let families = &bgp.groups[0].options.address_families;
        let unicast = &families[0];
        assert!(unicast.extended_nexthop_color);
        assert!(unicast.extended_nexthop_tunnel);
        assert_eq!(unicast.output_queue_priority.as_deref(), Some("priority 7"));
        assert_eq!(unicast.route_refresh_priority.as_deref(), Some("priority 8"));
        assert_eq!(unicast.withdraw_priority.as_deref(), Some("expedited"));

        let labeled = &families[1];
        assert!(!labeled.extended_nexthop_color);
        assert_eq!(labeled.output_queue_priority, None);
    }

    #[test]
    fn is_none_when_absent() {
        let config = juniper::parse_braces(
            "protocols {\n    bgp {\n        group G {\n            type external;\n        }\n    }\n}\n",
        );
        assert_eq!(bgp(&config).unwrap().output_queue_priority, None);
    }
}
