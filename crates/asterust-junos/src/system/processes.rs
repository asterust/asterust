//! Extracts the `processes` settings of `system`.
//!
//! Configuration path: `system { processes { ... } }`.
//!
//! # See also
//!
//! - [processes](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/processes-edit-system.html)

use crate::active_children;
use asterust::config::ConfigLine;

/// A process under `system processes`, e.g. `routing` or `dhcp-service`; `all-processes` appears
/// here too.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Process {
    pub name: String,
    /// `Some(true)` for `enable`, `Some(false)` for `disable`, `None` if neither is written.
    pub enabled: Option<bool>,
    /// `alternate-media` or `other-routing-engine`.
    pub failover: Option<String>,
    /// Seconds.
    pub timeout: Option<String>,
    pub no_livecore_dump_on_crash: bool,
}

/// Processes written either as `<name> disable;` or as `<name> { disable; failover ...; }`.
pub(super) fn parse_processes(processes: &ConfigLine) -> Vec<Process> {
    active_children(processes)
        .filter_map(|child| {
            let mut name_and_rest = child.text.split_whitespace();
            let name = name_and_rest.next()?.to_string();
            let mut words = name_and_rest.chain(active_children(child).flat_map(|c| c.text.split_whitespace()));
            let mut process = Process {
                name,
                enabled: None,
                failover: None,
                timeout: None,
                no_livecore_dump_on_crash: false,
            };
            while let Some(word) = words.next() {
                match word {
                    "enable" => process.enabled = Some(true),
                    "disable" => process.enabled = Some(false),
                    "failover" => process.failover = words.next().map(str::to_string),
                    "timeout" => process.timeout = words.next().map(str::to_string),
                    "no-livecore-dump-on-crash" => process.no_livecore_dump_on_crash = true,
                    _ => {}
                }
            }
            Some(process)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax page. See the README "Sources".
    const PROCESSES: &str = r#"
system {
    processes {
        dhcp-service disable;
        routing enable failover other-routing-engine;
        mib-process {
            enable;
            timeout 60;
            traceoptions {
                file mib.log;
                flag all;
            }
        }
        inactive: ntp disable;
        all-processes {
            no-livecore-dump-on-crash;
        }
    }
}
"#;

    #[test]
    fn extracts_processes_in_line_and_block_form() {
        let processes = system(&juniper::parse_braces(PROCESSES)).unwrap().processes;
        let names: Vec<_> = processes.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["dhcp-service", "routing", "mib-process", "all-processes"]);

        assert_eq!(processes[0].enabled, Some(false));
        assert_eq!(processes[1].enabled, Some(true));
        assert_eq!(processes[1].failover.as_deref(), Some("other-routing-engine"));
        assert_eq!(processes[2].enabled, Some(true));
        assert_eq!(processes[2].timeout.as_deref(), Some("60"));
    }

    #[test]
    fn extracts_process_flags() {
        let processes = system(&juniper::parse_braces(PROCESSES)).unwrap().processes;
        assert!(processes[3].no_livecore_dump_on_crash);
        assert_eq!(processes[3].enabled, None);
        assert!(!processes[0].no_livecore_dump_on_crash);
    }

    #[test]
    fn no_processes_is_empty() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.processes.is_empty());
    }
}
