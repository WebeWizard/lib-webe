<!--
SYNC IMPACT REPORT
==================
Version change: (template / unversioned) → 1.0.0
Rationale: Initial ratification of a concrete constitution from the template
  placeholder. MAJOR bump establishes the first governed baseline.

Modified principles:
  - [PRINCIPLE_1_NAME] → I. Code Quality (new)
  - [PRINCIPLE_2_NAME] → II. Testing Standards (NON-NEGOTIABLE) (new)
  - [PRINCIPLE_3_NAME] → III. User Experience Consistency (new)
  - [PRINCIPLE_4_NAME] → IV. Performance Requirements (new)
  - [PRINCIPLE_5_NAME] → removed (project requested four principles)

Added sections:
  - Additional Constraints (toolchain, workspace, and dependency rules)
  - Development Workflow & Quality Gates
  - Governance (concrete amendment/versioning policy)

Removed sections:
  - Fifth principle slot from the template (intentionally collapsed to four)

Templates requiring updates:
  - ✅ .specify/templates/plan-template.md (Constitution Check is generic; no
        edit required — gates derive from this file)
  - ✅ .specify/templates/spec-template.md (no constitution-specific references)
  - ✅ .specify/templates/tasks-template.md (task categories already cover
        testing, performance, and quality; no edit required)
  - ✅ .github/copilot-instructions.md (points to current plan; no edit required)

Follow-up TODOs:
  - None. RATIFICATION_DATE set to initial adoption date (today), since this is
    the first governed version.
-->

# Webe Toolkit Constitution

## Core Principles

### I. Code Quality

Code MUST be clear, idiomatic, and maintainable before it is clever.

- All code MUST pass `cargo fmt --check` and `cargo clippy` with no warnings on
  the default toolchain pinned in `rust-toolchain.toml`; warnings are treated as
  errors in CI.
- Public items in every crate (`webe_args`, `webe_auth`, `webe_log`, `webe_web`,
  and the `webe` facade) MUST carry doc comments describing intent, parameters,
  and failure modes.
- `unsafe` code is prohibited unless an inline comment justifies it and an
  integration or unit test exercises the invariant it relies on.
- Errors MUST be propagated with typed error values; `.unwrap()` and `.expect()`
  are forbidden in library code paths and permitted only in tests and examples.
- Modules MUST stay self-contained: a crate MUST NOT depend on another workspace
  crate solely for organizational convenience.

**Rationale**: The toolkit targets hobbyist developers building servers they must
maintain alone. Predictable, lint-clean, well-documented code lowers the cost of
ownership more than micro-optimized or terse implementations.

### II. Testing Standards (NON-NEGOTIABLE)

Every behavior change MUST be accompanied by tests that fail before the change
and pass after it.

- New public APIs and bug fixes MUST land with tests in the same change set;
  tests are written and observed to fail (Red) before implementation (Green).
- Each crate MUST keep unit tests beside its source and integration tests under
  its `tests/` directory; cross-crate behavior MUST be covered by the
  workspace-level `tests/` suite (e.g. `tests/combined.rs`, `tests/http.rs`).
- HTTP responders, authentication flows, and request/response parsing MUST have
  integration tests covering both success and failure paths.
- `cargo test --workspace` MUST pass; feature-gated crates such as `webe_auth`
  MUST be verified with their required features enabled (`--features auth`).
- A change MUST NOT be merged with failing, ignored, or commented-out tests
  unless an accompanying issue documents the deferral.

**Rationale**: Servers fail in production, not at compile time. Test-first
discipline and full-path coverage are the only reliable guard against regressions
in parsing, routing, and auth where subtle bugs become security issues.

### III. User Experience Consistency

The developer using the toolkit is the user; their experience MUST be consistent
across crates.

- Public APIs MUST follow consistent naming, error-handling, and async
  conventions across crates; a pattern established in one crate MUST be reused,
  not reinvented, in another.
- The `webe` facade MUST re-export crate functionality under stable, predictable
  module paths (`webe::web`, `webe::auth`, `webe::log`, `webe::args`), and
  feature flags MUST gate optional crates exactly as documented in the README.
- Error messages and logs surfaced to developers MUST be actionable: they MUST
  state what failed and, where possible, how to resolve it.
- Breaking changes to public APIs, module paths, or feature-flag behavior MUST be
  reflected in the README and require a MAJOR version bump of the affected crate.
- Examples under `examples/` MUST compile and demonstrate the documented happy
  path for any feature they showcase.

**Rationale**: A toolkit is only as good as its ergonomics. Consistency across
crates lets developers transfer knowledge from one module to the next and trust
that the facade behaves as advertised.

### IV. Performance Requirements

The toolkit MUST stay fast and stable enough for real server workloads on modest
hardware.

- Request handling in `webe_web` MUST avoid unnecessary allocations and blocking
  calls on async paths; blocking work MUST be moved off the async executor.
- Hot paths (request parsing, route matching, response encoding) MUST NOT
  introduce O(n²) or worse complexity over the size of a request or route table.
- Performance-sensitive changes MUST be justified with a benchmark or a
  measurement note in the change description; claimed improvements MUST be backed
  by data, not assertion.
- Resource usage MUST be bounded: streaming and chunked responses MUST be
  preferred over buffering entire payloads in memory for large bodies.
- A change MUST NOT regress measured throughput or latency of existing hot paths
  without explicit justification and sign-off.

**Rationale**: The project's stated goal is "fast and stable" servers. Without
explicit performance guardrails, convenience changes silently erode the core
value proposition on the limited hardware hobbyists typically run.

## Additional Constraints

- The pinned toolchain in `rust-toolchain.toml` and the edition declared in the
  workspace `Cargo.toml` are authoritative; changes to either require a
  constitution-aware review.
- New runtime dependencies MUST be justified by need, prefer well-maintained
  crates, and be added to `[workspace.dependencies]` for shared version control.
- Default builds MUST remain buildable without external system libraries; crates
  with external requirements (e.g. `webe_auth` needing MySQL client libraries)
  MUST stay behind opt-in feature flags.
- Secrets, credentials, and connection strings MUST NOT be committed; tests
  requiring them MUST read from environment (e.g. via `dotenvy`).

## Development Workflow & Quality Gates

- Every change MUST pass, before merge: `cargo fmt --check`, `cargo clippy`
  (no warnings), and `cargo test --workspace` including feature-gated suites
  relevant to the change.
- Code review MUST verify compliance with all four core principles; reviewers
  MUST block changes that add warnings, drop tests, break facade consistency, or
  regress performance without justification.
- The `/speckit.plan` Constitution Check gate MUST be evaluated against these
  principles before Phase 0 and re-checked after design.
- Complexity that violates a principle MUST be documented and justified in the
  plan's Complexity Tracking section, or the simpler approach MUST be adopted.

## Governance

This constitution supersedes other development practices for the Webe Toolkit
workspace. When guidance conflicts, the constitution wins.

- **Amendments**: Proposed via change request describing the rationale and
  impact. Amendments MUST update dependent templates and documentation in the
  same change set and record a Sync Impact Report at the top of this file.
- **Versioning policy**: This document follows semantic versioning.
  - MAJOR: Backward-incompatible governance changes or principle
    removals/redefinitions.
  - MINOR: A new principle or section, or materially expanded guidance.
  - PATCH: Clarifications, wording, and non-semantic refinements.
- **Compliance review**: All pull requests and reviews MUST verify compliance
  with these principles. Use the README and the current plan for runtime
  development guidance.

**Version**: 1.0.0 | **Ratified**: 2026-06-13 | **Last Amended**: 2026-06-13
