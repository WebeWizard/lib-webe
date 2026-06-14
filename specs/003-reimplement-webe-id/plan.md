# Implementation Plan: Reimplement Webe ID

**Branch**: `003-reimplement-webe-id` | **Date**: 2026-06-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/003-reimplement-webe-id/spec.md`

## Summary

Add WebeID generation to this workspace as a new `webe_id` crate surfaced through
the root `webe` facade. The implementation preserves the original WebeID concept:
a compact 64-bit identifier with 5 bytes of milliseconds since a custom epoch, 1
byte of node identity, and 2 bytes of sequence. The hot path is an O(1), bounded
state transition that does not allocate per ID, fails fast by default when one
node consumes all 65,536 sequence values in a millisecond, and exposes a separate
Tokio-friendly bounded backpressure path for server-style callers that choose to
wait safely.

The design emphasizes testability and explicit safety outcomes: deterministic
clock tests cover same-millisecond sequencing, time advancement, temporary clock
rewinds with catch-up recovery, restart safety from a persisted full last WebeID,
invalid node/input handling, and representable time-range exhaustion. Performance
work is reporting-only: the crate records repeatable self-contained generation,
concurrency, decomposition, and conversion measurements without comparison
benchmarks against the original repository and without pass/fail throughput or
latency gates.

## Technical Context

**Language/Version**: Rust (stable channel per `rust-toolchain.toml`, edition 2024, MSRV 1.85)

**Primary Dependencies**: Core generation uses the Rust standard library only.
The crate exposes optional Tokio-friendly bounded backpressure using the existing
workspace `tokio` dependency behind a feature. Performance reporting uses a
dev-only benchmark harness, planned as Criterion unless implementation finds a
lighter repeatable harness that gives equivalent reporting.

**Storage**: N/A inside the crate. Applications persist the full last generated
WebeID externally when they want restart safety.

**Testing**: `cargo test` unit tests in `crates/webe_id/src/`, integration tests
under `crates/webe_id/tests/`, facade coverage in workspace-level tests, Tokio
feature tests, and `cargo bench -p webe_id` for reporting-only measurements.

**Target Platform**: Cross-platform Rust library (Linux/macOS/Windows), usable in
Tokio-based server workloads when the Tokio feature is enabled.

**Project Type**: New Rust library crate within the existing Cargo workspace,
surfaced through the root `webe` facade.

**Performance Goals**: O(1) generation, decomposition, and byte conversion; no
per-ID heap allocation in default generation; bounded generator-owned memory;
repeatable self-contained throughput and latency reporting for single-generator,
concurrent, decomposition, and conversion paths. No comparison benchmark against
the original WebeID crate and no pass/fail throughput or latency gate.

**Constraints**: Preserve the 5-byte time, 1-byte node, 2-byte sequence layout;
default generation fails fast on per-millisecond sequence exhaustion; async
server-style waiting is separate and bounded; temporary clock rewind failures
recover after observed time catches up; restart safety derives from a persisted
full last WebeID; typed errors; no `.unwrap()`/`.expect()` in library paths; no
`unsafe`; all public items documented; lint-clean under `cargo fmt --check` and
`cargo clippy`.

**Scale/Scope**: Small performance-sensitive library crate: WebeID value type,
components/decomposition, node and epoch validation, generator state machine,
typed generation/input errors, optional Tokio backpressure helper, tests,
documentation, facade feature wiring, and benchmark reporting.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Evaluated against `.specify/memory/constitution.md` v1.0.0:

- **I. Code Quality**: PASS (planned). The new crate uses documented public
  types for WebeID values, components, generator configuration/state, and typed
  errors. Library paths propagate errors instead of panicking, avoid `unsafe`, and
  keep the crate self-contained rather than depending on another workspace crate
  for organization.
- **II. Testing Standards (NON-NEGOTIABLE)**: PASS (planned). Every spec behavior
  maps to automated tests in the same change set: layout/decomposition, ordering,
  same-millisecond sequencing, sequence reset, capacity failure, bounded async
  backpressure, restart marker validation, temporary clock rewind recovery,
  invalid input, concurrent uniqueness, facade reachability, examples, and
  benchmark-report generation. Tests use deterministic clock control where needed
  so boundary cases are reliable.
- **III. User Experience Consistency**: PASS (planned). The crate is reachable as
  `webe::id` behind a root facade feature, uses typed actionable errors with
  developer-facing messages, follows the workspace feature-flag pattern, and
  documents any intentional differences from the original WebeID repository in
  the crate README before release.
- **IV. Performance Requirements**: PASS (planned). Default generation is an O(1)
  bounded-state hot path with no per-ID heap allocation and no blocking wait.
  Async waiting is explicit and bounded. Performance-sensitive work is supported
  by self-contained benchmark reporting; future regressions can be compared to
  the reported baseline even though no pass/fail throughput gate is required for
  this feature.

Initial gate: **PASS**. No violations to justify; Complexity Tracking left empty.

Post-design re-check: **PASS**. The Phase 1 artifacts keep the same structure:
new focused crate, typed documented outcomes, deterministic test plan, facade
consistency, bounded hot paths, explicit optional async backpressure, and
reporting-only benchmark data. No constitution violations were introduced by
`research.md`, `data-model.md`, `contracts/public-api.md`, or `quickstart.md`.

Open design decisions resolved in `research.md`: crate/facade placement, layout
representation, optional Tokio backpressure, restart marker semantics, temporary
clock rewind recovery, deterministic test clock strategy, and performance
reporting policy.

## Project Structure

### Documentation (this feature)

```text
specs/003-reimplement-webe-id/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output - resolved design decisions
├── data-model.md        # Phase 1 output - entities and state transitions
├── quickstart.md        # Phase 1 output - runnable validation guide
├── contracts/
│   └── public-api.md    # Phase 1 output - public Rust API contract
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
Cargo.toml               # Add `crates/webe_id` workspace member, root `id`
                         # facade feature, optional `id-tokio` feature, and
                         # dev-only benchmark dependency wiring as needed
README.md                # Update Unique ID Generation section from external
                         # link to workspace crate usage and feature flags
src/
└── lib.rs               # Add `pub use webe_id as id` behind the `id` feature

crates/webe_id/
├── Cargo.toml           # New crate manifest; std-only core, optional Tokio
│                        # feature, dev benchmark harness
├── README.md            # Concepts, layout, examples, errors, compatibility
│                        # notes, benchmark reporting instructions
├── benches/
│   └── generation.rs    # Reporting-only benchmarks: generation, concurrency,
│                        # decomposition, storage conversions
├── src/
│   ├── lib.rs           # Crate docs, public exports, constants
│   ├── components.rs    # WebeID component view and validation helpers
│   ├── error.rs         # Typed creation/generation/parsing failures
│   ├── generator.rs     # Stateful generator and deterministic state machine
│   ├── id.rs            # WebeID value type and representations
│   ├── node.rs          # Node identifier validation/type
│   ├── time.rs          # Epoch, duration range, and clock abstraction
│   └── async_backpressure.rs # Feature-gated Tokio-friendly bounded wait path
└── tests/
    ├── generation.rs        # Layout, ordering, same-ms sequence, reset
    ├── safety.rs            # bad epoch, range exhaustion, restart, rewind,
    │                        # sequence exhaustion, invalid node/input
    ├── representations.rs   # u64, big-endian bytes, decimal, hex round trips
    ├── concurrency.rs       # documented shared usage and zero duplicates
    └── tokio_backpressure.rs# feature-gated bounded async behavior

tests/
└── id.rs                # Facade-level reachability through `webe::id`
```

**Structure Decision**: Add a new single-purpose `crates/webe_id` Rust library
crate rather than folding ID generation into `webe_web` or the root facade. The
crate owns WebeID semantics and tests; the root `webe` crate only exposes it via
feature-gated re-export. Core generation stays standard-library-only, while the
Tokio-friendly bounded backpressure path is feature-gated inside the same crate
so async server users get an ergonomic path without forcing Tokio into the core
hot path.

## Complexity Tracking

> No constitution violations. Section intentionally empty.
