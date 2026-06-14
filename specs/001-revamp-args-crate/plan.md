# Implementation Plan: Revamp Args Crate

**Branch**: `001-revamp-args-crate` | **Date**: 2026-06-13 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/001-revamp-args-crate/spec.md`

## Summary

Revamp the `webe_args` crate into a well-tested, well-documented parser for simple
command-line options. The crate lets developers declare named options (long name,
optional short alias, description, required/optional, flag/value, optional value
validation), then validate a caller-supplied command line in one pass that either
confirms success or returns all discovered failures in deterministic order. The
technical approach keeps the crate dependency-free, accepts an explicit token
slice (instead of reading the live process environment) so behavior is
deterministic and testable, and replaces the existing `panic!`-based
`parse_args` with typed errors per the constitution.

## Technical Context

**Language/Version**: Rust (stable channel per `rust-toolchain.toml`, edition 2024, MSRV 1.85)

**Primary Dependencies**: None (standard library only; the crate stays dependency-free)

**Storage**: N/A

**Testing**: `cargo test` — unit tests in `src/`, integration tests under `crates/webe_args/tests/`

**Target Platform**: Cross-platform Rust library (Linux/macOS/Windows)

**Project Type**: Single Rust library crate within the existing Cargo workspace

**Performance Goals**: Validate a command line of ~20 declared options and ~100 tokens with no user-noticeable startup delay (SC-005); no worse than linear complexity over option count and token count

**Constraints**: No `.unwrap()`/`.expect()` in library paths; typed errors; no `unsafe`; lint-clean under `cargo fmt --check` and `cargo clippy`; default build requires no external system libraries; public items carry doc comments

**Scale/Scope**: Small crate — one public option-definition type, one parser/registry type, one error/failure type, and a validation report; surfaced through the `webe::args` facade re-export

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Evaluated against `.specify/memory/constitution.md` v1.0.0:

- **I. Code Quality**: PASS (planned). Public items get doc comments (intent, params,
  failure modes). Errors are typed (`ArgError` revamp), removing the current
  `panic!` in `parse_args` from library paths. No `unsafe`. The crate stays
  self-contained with no workspace-internal deps. Plan targets `cargo fmt`/`clippy`
  clean.
- **II. Testing Standards (NON-NEGOTIABLE)**: PASS (planned). Every behavior in the
  spec maps to tests (FR-015). Tests are added in the same change set and use the
  standard harness against caller-supplied tokens, so `cargo test --workspace` runs
  them without manual command-line text (FR-016) — this fixes the current
  `harness = false` / `required-features` manual test that cannot run by default.
- **III. User Experience Consistency**: PASS (planned). Errors are actionable and
  name the offending option/token. The crate remains reachable via the
  `webe::args` facade re-export. Breaking API changes are documented in the crate
  README and surfaced before release (spec Assumptions).
- **IV. Performance Requirements**: PASS (planned). Validation is single-pass and
  linear over options/tokens; no O(n^2) scans. No allocation-heavy hot paths beyond
  building the failure list.

Initial gate: **PASS**. No violations to justify; Complexity Tracking left empty.

## Project Structure

### Documentation (this feature)

```text
specs/001-revamp-args-crate/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   └── public-api.md    # Public API contract for the crate
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
crates/webe_args/
├── Cargo.toml           # Drop manual-cli-test feature & harness=false test wiring
├── README.md            # Expanded usage + behavior documentation
├── src/
│   └── lib.rs           # Public API: option definition, parser/registry, errors,
│                        # validation report, plus #[cfg(test)] unit tests
└── tests/
    └── cli.rs           # Standard-harness integration tests over supplied tokens
```

**Structure Decision**: Single library crate. The revamp stays inside the existing
`crates/webe_args/` directory and the workspace facade re-export at
`src/lib.rs` (`pub use webe_args as args`). Unit tests live beside the source in
`src/lib.rs` under `#[cfg(test)]`; integration tests covering parsing and every
documented failure path live in `crates/webe_args/tests/cli.rs` using the default
test harness so `cargo test` runs them without custom invocation arguments.

## Complexity Tracking

> No constitution violations. Section intentionally empty.
