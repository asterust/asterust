# Agent Guidelines for asterust

`asterust` is a small, pure-Rust library: vendor-namespaced hierarchical
configuration parsing (`config::cisco`, `config::juniper`). It has no
PyO3/binding dependency of its own — consumers (e.g. a PyO3 extension
module) add their own bindings on top. These guidelines apply to this
crate specifically; prune/extend them if the crate's scope changes.

## Core Principle

Write clear, correct, idiomatic Rust. Prioritize clarity and
maintainability over cleverness — this is a library other code will
depend on, so predictable, well-tested behavior matters more than
micro-optimization.

## Code Style and Formatting

- Use `rustfmt` (config in `rustfmt.toml`, `max_width = 120` to match
  the `ruff line-length = 120` convention used in sibling projects).
- Use meaningful, descriptive names. `snake_case` for
  functions/variables/modules, `PascalCase` for types/traits,
  `SCREAMING_SNAKE_CASE` for constants.
- Follow idiomatic Rust / the Rust API Guidelines.
- No emoji or unicode that emulates emoji, except when a test is
  specifically exercising multibyte/unicode handling.
- Avoid redundant comments that just restate what the next line does.
  A comment should explain *why*, not *what*, when the *what* is
  already obvious from the code.

## Documentation

- Public functions, structs, enums, and methods should have doc
  comments explaining purpose, non-obvious parameters, and return
  value. Match the existing style in this crate (see `src/config/mod.rs`,
  `src/config/cisco.rs`) rather than a generic template — most functions
  here need only 1-3 lines, not full `# Arguments`/`# Returns` sections.
- Keep doc comments up to date with the code; stale docs are worse
  than no docs.

## Error Handling

- Prefer `Option<T>` for "not found" / "didn't match" cases — this is
  the common case in a parser (e.g. `find_child_with`, `find_line_with`,
  `match_str_group` all return `Option`, not an error).
- Reserve `Result<T, E>` for operations that can genuinely fail with
  information the caller needs to act on. None of the current public
  API needs this; don't introduce `thiserror`/`anyhow` speculatively —
  add them only when a real fallible operation needs them.
- `.unwrap()` is acceptable only on a compile-time-constant invariant
  (e.g. `Regex::new("...a hardcoded, known-valid pattern...").unwrap()`).
  Prefer `.expect("why this can't fail")` over bare `.unwrap()` in new
  code so a panic (if the invariant is ever violated by a future edit)
  is self-explanatory.
- Never `.unwrap()` on anything derived from external/caller-supplied
  input.

## Function and Type Design

- Keep functions focused on a single responsibility.
- Prefer borrowing (`&T`, `&str`) over taking ownership when the
  function doesn't need it.
- Return early to reduce nesting.
- Prefer iterator adapters over manual loops where it's at least as
  clear (don't force it when a plain loop reads better).
- Derive `Debug`, `Clone`, `PartialEq`/`Eq` where appropriate (see
  `ConfigLine`).
- Public fields are fine where the type is a plain data holder with no
  invariants to protect (e.g. `ConfigLine.text`, `ConfigBlock.root_line`,
  `Config.blocks`) — don't add accessor boilerplate for its own sake.

## Testing

- Unit tests (`#[cfg(test)] mod tests` inline in the same file) for
  anything that needs access to private helpers — most tests in this
  crate are this kind.
- Black-box integration tests (`tests/*.rs`, a separate crate, `pub`
  API only) for end-to-end scenarios a real consumer would hit — see
  `tests/config_cisco.rs`, `tests/config_juniper.rs`.
- New parsing behavior needs a test with realistic input (a real-ish
  config/capture snippet), not just a trivial one-liner.
- Don't commit commented-out tests.

## Imports and Dependencies

- No wildcard imports except `use super::*;` in test modules.
- Keep dependencies minimal — this crate exists partly *because*
  pulling in a huge dependency tree for simple text parsing is
  wasteful. Justify any new dependency in the PR description.
- New dependencies need a version constraint in `Cargo.toml`.

## Version Control

- Clear, descriptive commit messages (explain *why* a change was made,
  not just *what* changed — the diff already shows the *what*).
- Never commit commented-out code, debug `println!`/`dbg!` calls, or
  credentials.
- Bump `Cargo.toml`'s `version` whenever a branch adds a new feature
  (new module, new public API, new vendor/syntax support) — minor bump
  for features, patch for pure bugfixes. Don't bump for docs/tooling/
  test-only changes.

## Before Committing

- [ ] `cargo test` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo fmt --all -- --check` passes
- [ ] New public items have doc comments
- [ ] No commented-out code, debug prints, or credentials
- [ ] `Cargo.toml` version bumped if this change is a new feature

These are exactly the checks `.github/workflows/ci.yml` runs — if all
four pass locally, CI will pass.
