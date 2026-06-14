# Phase 0 Research: Revamp Args Crate

All five clarifications were resolved in the spec (see `spec.md` → Clarifications),
so there are no open `NEEDS CLARIFICATION` items. This document records the
technical decisions that shape Phase 1 design.

## Decision: Dependency-free, standard-library-only implementation

- **Decision**: Implement the parser using only `std`; add no runtime dependencies.
- **Rationale**: The constitution requires default builds with no external system
  libraries and justification for new dependencies. Simple option parsing needs
  only string comparison and collections, so a dependency (e.g. `clap`) would add
  weight without value and conflicts with the "simple options" scope.
- **Alternatives considered**: `clap`/`pico-args`/`argh` — rejected as over-scoped
  or as replacing the crate's own purpose; they also pull in derive macros or
  larger APIs than the simple-option goal requires.

## Decision: Parse caller-supplied tokens instead of reading `env::args()` directly

- **Decision**: The validation/parse entry point accepts an explicit sequence of
  tokens (e.g. `&[String]` or an iterator). A thin convenience path may read the
  process arguments, but core logic operates on supplied tokens.
- **Rationale**: FR-002 and FR-016 require repeatable tests through the normal
  `cargo test` flow without manually passing command-line text. The current crate
  calls `env::args()` inside `get`, which forces the existing `tests/cli.rs` to use
  `harness = false` and a `required-features` flag — it cannot run by default. A
  token slice makes every scenario a plain deterministic unit/integration test.
- **Alternatives considered**: Keep reading `env::args()` — rejected because it
  blocks deterministic testing and violates Testing Standards. A global/thread-local
  override — rejected as hidden state and harder to reason about.

## Decision: Typed errors and an all-failures validation report

- **Decision**: Replace the `panic!` in `parse_args` with typed error values.
  Reading a single option returns a typed result; full startup validation returns
  success or a collection of all discovered failures in deterministic order.
- **Rationale**: Constitution Code Quality forbids `panic!`/`unwrap` in library
  paths and requires typed errors. The clarification chose "return all discovered
  failures in deterministic order" (FR-010), so the report holds an ordered list.
- **Alternatives considered**: First-failure-only — rejected by clarification.
  Panic/process-exit inside the library — rejected by constitution; presentation
  belongs to the calling application.

## Decision: Strict handling of unknown options, duplicates, dash-values, positionals

- **Decision**: During full startup validation, reject unknown option tokens,
  duplicate option occurrences, and bare positional tokens; a `-`-prefixed token
  after a value option is treated as an option token (the value option then fails
  missing-value).
- **Rationale**: These match the recorded clarifications (FR-006, FR-010, FR-011,
  FR-012) and keep the simple-option contract predictable and easy to document.
  stdin-based piping is unaffected because piped data arrives on stdin, not as
  argument tokens.
- **Alternatives considered**: Lenient/ignore modes and configurable strictness —
  rejected to keep the first revamp scope small and behavior unambiguous.

## Decision: Deterministic ordering rule for the failure list

- **Decision**: Failures are ordered first by definition-level problems
  (conflicting option definitions) detected before input parsing, then by
  left-to-right position of offending tokens in the supplied command line; for a
  missing required option with no token, order by option registration order.
- **Rationale**: SC-003 and FR-010 require deterministic order so tests can assert
  exact failure sequences and developers get stable output.
- **Alternatives considered**: Unordered/`HashMap` iteration — rejected as
  non-deterministic and untestable.

## Decision: Validation strategy and coverage

- **Decision**: Unit tests in `src/lib.rs` (`#[cfg(test)]`) cover option-definition
  validation and single-option reads; integration tests in
  `crates/webe_args/tests/cli.rs` (default harness) cover full-command-line
  validation, every failure category, aliases, flags, dash-values, duplicates,
  positionals, and all-failures ordering. README + rustdoc examples are kept in
  sync and exercised by `cargo test` doctests where practical.
- **Rationale**: Satisfies FR-013/FR-015/FR-016 and Testing Standards; doctests
  keep documentation examples (SC-004) from drifting.
- **Alternatives considered**: Keeping the manual `harness = false` test —
  rejected because it cannot run under the normal verification flow.
