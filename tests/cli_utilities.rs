//! Integration tests: exercise the public CLI-capture utilities
//! (`prompt`, `date`, `clean`, `paths`) the way a capture-parsing consumer
//! (e.g. netconfig-splitter) would use them together.

use asterust::clean::clean_line;
use asterust::date::{try_parse_date, CaptureDate};
use asterust::paths::sanitize_path_component;
use asterust::prompt::{match_prompt, normalize_command};

#[test]
fn recognizes_prompt_and_normalizes_abbreviated_command() {
    let prompt = match_prompt("RP/0/RSP1/CPU0:router1#sh run").expect("prompt line");
    assert_eq!(prompt.hostname, "router1");
    assert_eq!(normalize_command(&prompt.command), "show run");
}

#[test]
fn recognizes_asa_multi_context_prompt() {
    let prompt = match_prompt("ciscoasa/admin#show running-config").expect("asa prompt");
    assert_eq!(prompt.hostname, "ciscoasa/admin");
    assert_eq!(prompt.command, "show running-config");
}

#[test]
fn cleans_terminal_artifacts_before_date_parsing() {
    // A banner line with a trailing pagination prompt artifact spliced in,
    // as can happen with some terminal capture tools.
    let raw = "Mon Oct 19 02:45:08.742 UTC\x1b[0m";
    let cleaned = clean_line(raw).expect("not a pagination prompt");
    let parsed = try_parse_date(&cleaned, 2026).expect("date banner");
    assert_eq!(
        parsed.date(),
        CaptureDate {
            year: 2026,
            month: 10,
            day: 19
        }
    );
}

#[test]
fn drops_pagination_prompt_lines() {
    assert_eq!(clean_line("  --More--  "), None);
}

#[test]
fn sanitizes_hostname_and_command_into_a_safe_output_path() {
    let prompt = match_prompt("switch1/eng#show interfaces").expect("prompt line");
    let host_dir = sanitize_path_component(&prompt.hostname);
    let command_dir = sanitize_path_component(&normalize_command(&prompt.command));
    assert_eq!(host_dir, "switch1_eng");
    assert_eq!(command_dir, "show interfaces");
}
