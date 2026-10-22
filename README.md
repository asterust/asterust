# asterust

[![CI](https://github.com/asterust/asterust/actions/workflows/ci.yml/badge.svg)](https://github.com/asterust/asterust/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/asterust.svg)](https://crates.io/crates/asterust)
[![docs.rs](https://docs.rs/asterust/badge.svg)](https://docs.rs/asterust)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

Reusable parsing primitives for network device CLI output: prompts,
timestamp banners, terminal artifact cleanup, path sanitization, and
hierarchical configuration parsing.

Pure Rust, with no PyO3 (or any other binding) dependency, so any Python
extension module — or non-Python tool — can depend on it directly and add
its own bindings on top.

## Modules

- **`prompt`** — recognizing device prompts (Cisco IOS-XR's
  `RP/0/RSP1/CPU0:host#` style, generic IOS/IOS-XE/NX-OS/ASA prompts,
  including ASA multi-context suffixes) and normalizing common command
  abbreviations (`sh` → `show`, etc.).
- **`date`** — parsing timestamp banners: IOS-XR's year-less
  auto-printed per-command banner, and the year-bearing timestamp from
  `show clock` on IOS/IOS-XE/NX-OS.
- **`clean`** — stripping terminal capture artifacts (ANSI escapes,
  backspace erasures, `--More--` pagination prompts) out of raw lines.
- **`paths`** — making arbitrary strings safe to use as filesystem path
  segments.
- **`config`** — a vendor-agnostic hierarchical configuration tree
  (`ConfigLine` / `ConfigBlock` / `Config`), with a parsing strategy per
  syntax family:
  - `config::cisco` — indentation-based hierarchy, `!`-terminated blocks
    (IOS, IOS-XE, IOS-XR, NX-OS, and similar CLIs).
  - `config::juniper` — brace-delimited hierarchy, `;`-terminated leaves
    (JunOS `show configuration` output).

## Usage

```toml
[dependencies]
asterust = { path = "../asterust" }  # or a git/crates.io dependency once published
```

```rust
use asterust::config::cisco;

let config = cisco::parse(raw_running_config_text);
let ospf_blocks = config.find_blocks_by_pattern(r"router ospf");
```

## Development

```sh
make test   # unit + integration tests
make lint   # clippy, warnings as errors
make fmt    # format with rustfmt
make check  # everything CI runs (fmt --check, lint, test)
```
