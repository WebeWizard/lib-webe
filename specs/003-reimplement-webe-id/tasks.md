# Tasks: Reimplement Webe ID

**Input**: Design documents from `/specs/003-reimplement-webe-id/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/public-api.md`, `quickstart.md`

**Tests**: Included. The feature specification and constitution require automated coverage for every documented behavior, with tests written before implementation.

**Organization**: Tasks are grouped by user story so each story can be implemented and tested as an independent increment once its listed dependencies are satisfied.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it touches different files and has no dependency on incomplete tasks in the same phase
- **[Story]**: User story label for story phases only (`US1`, `US2`, `US3`, `US4`)
- Every task includes the exact repository path to create or modify

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add the new workspace crate, crate skeleton, and empty verification surfaces.

- [X] T001 Add `crates/webe_id` to workspace members and default members in `Cargo.toml`
- [X] T002 Create new crate manifest with optional `tokio` feature and benchmark target in `crates/webe_id/Cargo.toml`
- [X] T003 Create source module scaffold in `crates/webe_id/src/lib.rs`, `crates/webe_id/src/components.rs`, `crates/webe_id/src/error.rs`, `crates/webe_id/src/generator.rs`, `crates/webe_id/src/id.rs`, `crates/webe_id/src/node.rs`, `crates/webe_id/src/time.rs`, and `crates/webe_id/src/async_backpressure.rs`
- [X] T004 [P] Create initial crate README scaffold in `crates/webe_id/README.md`
- [X] T005 [P] Create benchmark scaffold in `crates/webe_id/benches/generation.rs`
- [X] T006 [P] Create integration test scaffolds in `crates/webe_id/tests/generation.rs`, `crates/webe_id/tests/safety.rs`, `crates/webe_id/tests/representations.rs`, `crates/webe_id/tests/concurrency.rs`, and `crates/webe_id/tests/tokio_backpressure.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Establish shared API boundaries, errors, time control, and generator construction pieces needed by all stories.

**Critical**: No user story work should begin until this phase is complete.

- [X] T007 Configure crate-level docs, `missing_docs`, `unsafe_code`, and public module exports in `crates/webe_id/src/lib.rs`
- [X] T008 [P] Define typed error enums, categories, `Display`, and `std::error::Error` implementations in `crates/webe_id/src/error.rs`
- [X] T009 [P] Define `NodeId` validation and conversion API for `0..=255` values in `crates/webe_id/src/node.rs`
- [X] T010 [P] Define epoch constants, 40-bit millisecond range helpers, and system/test clock abstraction in `crates/webe_id/src/time.rs`
- [X] T011 [P] Add deterministic fixed and advancing clock helpers for integration tests in `crates/webe_id/tests/common/mod.rs`
- [X] T012 Define generator configuration, optional restart marker inputs, and initial state fields in `crates/webe_id/src/generator.rs`

**Checkpoint**: The crate compiles as an empty public surface with documented types and shared test helpers.

---

## Phase 3: User Story 1 - Generate WebeIDs With Preserved Semantics (Priority: P1) MVP

**Goal**: Generate compact sortable 64-bit WebeIDs with 5-byte time, 1-byte node, and 2-byte sequence components.

**Independent Test**: Configure a deterministic epoch and node, generate IDs across controlled milliseconds, then confirm uniqueness, time ordering, same-millisecond sequence advancement, sequence reset after time advancement, and decomposition into expected components.

### Tests for User Story 1

Write these tests first and confirm they fail before implementing the story.

- [X] T013 [US1] Write failing layout, 10,000-sample time-ordering, same-millisecond sequence, node differentiation, and sequence reset tests in `crates/webe_id/tests/generation.rs`
- [X] T014 [P] [US1] Write failing numeric, big-endian byte, decimal text, and hexadecimal text round-trip tests in `crates/webe_id/tests/representations.rs`

### Implementation for User Story 1

- [X] T015 [US1] Implement `WebeId` raw value type, 40/8/16-bit masks, shifts, ordering, and constructors in `crates/webe_id/src/id.rs`
- [X] T016 [US1] Implement `WebeIdComponents` decomposition and recomposition API in `crates/webe_id/src/components.rs`
- [X] T017 [US1] Implement default generator sequencing, same-millisecond advancement, and sequence reset on time advancement in `crates/webe_id/src/generator.rs`
- [X] T018 [US1] Implement stable numeric, big-endian byte, decimal text, and hexadecimal text conversions in `crates/webe_id/src/id.rs`
- [X] T019 [US1] Export `WebeId`, `WebeIdComponents`, `NodeId`, `Generator`, constants, and core errors from `crates/webe_id/src/lib.rs`

**Checkpoint**: User Story 1 is complete when `cargo test -p webe_id --test generation` and `cargo test -p webe_id --test representations` pass for the core layout and representation behavior.

---

## Phase 4: User Story 2 - Protect Uniqueness During Restarts and Clock Problems (Priority: P2)

**Goal**: Refuse to generate duplicate-risk IDs under restart, clock rewind, invalid input, exhausted time range, and sequence capacity boundaries.

**Independent Test**: Use deterministic clocks and persisted full WebeIDs to verify safe restart, unsafe restart rejection, clock rewind temporary failure and recovery, bad epoch/range failures, invalid node/input failures, and capacity exhaustion fail-fast behavior.

### Tests for User Story 2

Write these tests first and confirm they fail before implementing the story.

- [X] T020 [US2] Write failing tests for bad epoch, exhausted time range, invalid node, restart marker node/time safety, temporary clock rewind recovery, and sequence capacity exhaustion in `crates/webe_id/tests/safety.rs`

### Implementation for User Story 2

- [X] T021 [US2] Implement custom epoch validation and maximum representable time range checks in `crates/webe_id/src/time.rs`
- [X] T022 [US2] Implement full WebeID restart marker parsing, node compatibility, and current-duration safety checks in `crates/webe_id/src/generator.rs`
- [X] T023 [US2] Implement temporary clock rewind failure and catch-up recovery in `crates/webe_id/src/generator.rs`
- [X] T024 [US2] Implement default sequence capacity fail-fast behavior after 65,536 IDs in one millisecond in `crates/webe_id/src/generator.rs`
- [X] T025 [US2] Wire bad epoch, exhausted range, bad restart marker, clock rewind, capacity, invalid node, and malformed input outcomes to typed errors in `crates/webe_id/src/error.rs`
- [X] T026 [US2] Implement malformed decimal, hexadecimal, and byte input parsing failures in `crates/webe_id/src/id.rs`

**Checkpoint**: User Story 2 is complete when `cargo test -p webe_id --test safety` passes and every documented safety outcome can be matched independently.

---

## Phase 5: User Story 3 - Meet Server-Workload Performance Expectations (Priority: P3)

**Goal**: Support documented high-volume and async/server-style usage with bounded resource behavior and repeatable performance reporting.

**Independent Test**: Generate at least 100,000 IDs through the documented concurrent pattern with zero duplicates, validate bounded memory behavior during at least 1,000,000 generated IDs, verify normal Tokio request-style generation does not use blocking waits, verify Tokio bounded backpressure success and timeout/capacity paths, and produce reporting-only benchmark output.

### Tests for User Story 3

Write these tests first and confirm they fail before implementing the story.

- [X] T027 [US3] Write failing concurrent uniqueness and volume bounded-memory tests in `crates/webe_id/tests/concurrency.rs`
- [X] T028 [P] [US3] Write failing Tokio normal-load no-wait request-style tests plus bounded backpressure success and timeout/capacity tests in `crates/webe_id/tests/tokio_backpressure.rs`
- [X] T029 [P] [US3] Add reporting benchmark scenarios for generation, concurrency, decomposition, and conversion in `crates/webe_id/benches/generation.rs`

### Implementation for User Story 3

- [X] T030 [US3] Ensure `Generator` uses fixed bounded state and document shared synchronization with no lock held across async `.await` points in `crates/webe_id/src/generator.rs`
- [X] T031 [US3] Implement feature-gated bounded async backpressure generation in `crates/webe_id/src/async_backpressure.rs`
- [X] T032 [US3] Expose the Tokio backpressure API only behind the `tokio` feature in `crates/webe_id/src/lib.rs`
- [X] T033 [US3] Complete benchmark reporting with throughput, p95 latency where practical, duplicate-rate observations, feature flags, OS, hardware, and toolchain context in `crates/webe_id/benches/generation.rs`
- [X] T034 [US3] Document normal async no-wait behavior and bounded backpressure limits in `crates/webe_id/src/async_backpressure.rs`

**Checkpoint**: User Story 3 is complete when `cargo test -p webe_id --test concurrency`, `cargo test -p webe_id --features tokio --test tokio_backpressure`, and `cargo bench -p webe_id` produce the documented outcomes.

---

## Phase 6: User Story 4 - Understand and Validate the WebeID Model (Priority: P4)

**Goal**: Make the WebeID model, facade usage, uniqueness domain, limits, errors, performance profile, and compatibility notes understandable from docs and examples.

**Independent Test**: From a clean checkout, follow docs to generate an ID, decompose it, persist a restart marker, use the facade, review boundary/failure behavior, and run the documented verification suite in under 15 minutes.

### Tests for User Story 4

Write these tests first and confirm they fail before implementing the story.

- [X] T035 [P] [US4] Write failing facade reachability tests for `webe::id` with `id` and `id-tokio` features in `tests/id.rs`
- [X] T036 [US4] Add compile-checked rustdoc examples for generation, decomposition, restart safety, concurrent usage, and failure handling in `crates/webe_id/src/lib.rs`

### Implementation for User Story 4

- [X] T037 [US4] Add root optional `webe_id` dependency plus `id` and `id-tokio` facade features in `Cargo.toml`
- [X] T038 [US4] Add `pub use webe_id as id` behind the `id` feature in `src/lib.rs`
- [X] T039 [P] [US4] Update the root Unique ID Generation section with facade feature flags and usage in `README.md`
- [X] T040 [P] [US4] Write crate README guide covering layout, uniqueness domain, restart markers, capacity limits, errors, benchmarks, and compatibility notes in `crates/webe_id/README.md`
- [X] T041 [US4] Add rustdoc examples for `WebeId` and `WebeIdComponents` in `crates/webe_id/src/id.rs` and `crates/webe_id/src/components.rs`
- [X] T042 [US4] Add rustdoc examples for `Generator`, `NodeId`, and typed errors in `crates/webe_id/src/generator.rs`, `crates/webe_id/src/node.rs`, and `crates/webe_id/src/error.rs`

**Checkpoint**: User Story 4 is complete when `cargo test -p webe --features id,id-tokio`, `cargo test -p webe_id --doc`, and a documentation review find no undocumented public outcomes.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final quality gates, verification commands, and release readiness.

- [X] T043 Run formatting and fix rustfmt issues in `crates/webe_id/src/lib.rs`, `crates/webe_id/src/components.rs`, `crates/webe_id/src/error.rs`, `crates/webe_id/src/generator.rs`, `crates/webe_id/src/id.rs`, `crates/webe_id/src/node.rs`, `crates/webe_id/src/time.rs`, `crates/webe_id/src/async_backpressure.rs`, `crates/webe_id/tests/generation.rs`, `crates/webe_id/tests/safety.rs`, `crates/webe_id/tests/representations.rs`, `crates/webe_id/tests/concurrency.rs`, `crates/webe_id/tests/tokio_backpressure.rs`, `src/lib.rs`, and `Cargo.toml`
- [X] T044 Run clippy and fix warnings for the direct crate in `crates/webe_id/src/lib.rs`, `crates/webe_id/src/components.rs`, `crates/webe_id/src/error.rs`, `crates/webe_id/src/generator.rs`, `crates/webe_id/src/id.rs`, `crates/webe_id/src/node.rs`, `crates/webe_id/src/time.rs`, and `crates/webe_id/src/async_backpressure.rs`
- [X] T045 Run direct, Tokio-feature, facade, and workspace tests excluding `webe_auth` as needed, then fix feature wiring issues in `Cargo.toml` and `src/lib.rs`
- [X] T046 Run benchmark reporting and record benchmark command/context guidance in `crates/webe_id/README.md`
- [X] T047 Audit library code for forbidden `unsafe`, `.unwrap()`, `.expect()`, `dbg!`, and `println!` usage in `crates/webe_id/src/lib.rs`, `crates/webe_id/src/components.rs`, `crates/webe_id/src/error.rs`, `crates/webe_id/src/generator.rs`, `crates/webe_id/src/id.rs`, `crates/webe_id/src/node.rs`, `crates/webe_id/src/time.rs`, and `crates/webe_id/src/async_backpressure.rs`
- [X] T048 Update validation commands and expected outcomes if implementation details changed in `specs/003-reimplement-webe-id/quickstart.md`

---

## Dependencies & Execution Order

### Phase Dependencies

Setup (Phase 1) has no dependencies and can start immediately.

Foundational (Phase 2) depends on Phase 1 and blocks all user stories.

User Story 1 (Phase 3) depends on Phase 2 and is the MVP.

User Story 2 (Phase 4) depends on the core value/generator behavior from User Story 1 because restart markers, clock safety, and capacity safety operate on real WebeIDs.

User Story 3 (Phase 5) depends on User Story 1 for generation semantics and on User Story 2 for capacity outcomes used by bounded backpressure.

User Story 4 (Phase 6) can begin documentation scaffolding after Phase 2, but final facade, docs, and examples should be completed after User Stories 1-3 stabilize.

Polish (Phase 7) depends on all desired user stories being complete.

### User Story Dependencies

US1 (P1) -> no story dependencies after Foundational.

US2 (P2) -> depends on US1 value and generator behavior.

US3 (P3) -> depends on US1 generation semantics and US2 capacity/error outcomes.

US4 (P4) -> documentation and facade tests can start early; final completion depends on US1-US3 behavior.

### Within Each User Story

Tests are written first and observed failing before implementation.

Value models and component types come before generator behavior.

Generator behavior comes before facade and documentation examples that rely on it.

Benchmarks are reporting-only and must not introduce pass/fail throughput or latency gates.

---

## Parallel Opportunities

Setup tasks T004, T005, and T006 can run in parallel after T001-T003 are understood because they touch README, benchmark, and test files.

Foundational tasks T008, T009, T010, and T011 can run in parallel because they touch separate error, node, time, and test helper files.

US1 tests T013 and T014 can be written in parallel because they touch separate generation and representation test files.

US3 tests T027, T028, and T029 can be written in parallel because they touch concurrency tests, Tokio tests, and benchmarks.

US4 documentation tasks T039 and T040 can run in parallel with rustdoc examples once the public names are stable.

---

## Parallel Example: User Story 1

```text
Task: T013 Write failing layout and sequencing tests in crates/webe_id/tests/generation.rs
Task: T014 Write failing representation round-trip tests in crates/webe_id/tests/representations.rs
```

---

## Parallel Example: User Story 2

```text
Task: T021 Implement epoch and range checks in crates/webe_id/src/time.rs
Task: T025 Wire typed safety errors in crates/webe_id/src/error.rs
```

---

## Parallel Example: User Story 3

```text
Task: T027 Write concurrent and volume tests in crates/webe_id/tests/concurrency.rs
Task: T028 Write Tokio backpressure tests in crates/webe_id/tests/tokio_backpressure.rs
Task: T029 Add reporting benchmark scenarios in crates/webe_id/benches/generation.rs
```

---

## Parallel Example: User Story 4

```text
Task: T035 Write facade reachability tests in tests/id.rs
Task: T039 Update root README Unique ID Generation docs in README.md
Task: T040 Write crate README guide in crates/webe_id/README.md
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Stop and validate `cargo test -p webe_id --test generation` and `cargo test -p webe_id --test representations`.
5. Demo direct `webe_id` generation, ordering, decomposition, and representation round trips.

### Incremental Delivery

1. Deliver Setup and Foundational tasks so the crate compiles and exposes shared types.
2. Deliver US1 as the MVP WebeID generator with preserved layout and semantics.
3. Deliver US2 to harden restart, clock, input, range, and capacity safety.
4. Deliver US3 to validate server-workload behavior, optional Tokio backpressure, and benchmark reporting.
5. Deliver US4 to finalize facade reachability, docs, examples, compatibility notes, and quickstart validation.

### Validation Commands

```bash
cargo fmt --check
cargo clippy -p webe_id --all-targets -- -D warnings
cargo test -p webe_id
cargo test -p webe_id --features tokio
cargo test -p webe --features id,id-tokio
cargo test --workspace --exclude webe_auth
cargo bench -p webe_id
```

Expected outcome: all checks pass, with benchmark output used for reporting only and no comparison against the original WebeID repository.