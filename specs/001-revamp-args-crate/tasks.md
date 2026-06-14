---
description: "Task list for revamping the webe_args crate"
---

# Tasks: Revamp Args Crate

**Input**: Design documents from `/specs/001-revamp-args-crate/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/public-api.md, quickstart.md

**Intended implementer**: The **Modern Rust** agent. Apply edition 2024 idioms, typed
errors (no `panic!`/`unwrap`/`expect` in library paths), no `unsafe`, and keep the
crate dependency-free (`std` only). Run `cargo fmt --check` and
`cargo clippy -p webe_args -- -D warnings` after edits.

**Tests**: Tests are REQUIRED for this feature (FR-015, FR-016, SC-002). Every
documented behavior maps to at least one automated test runnable via `cargo test`
without manual command-line input.

**Test-first (Constitution II, NON-NEGOTIABLE)**: Within every user story, the test
tasks are listed FIRST and MUST be authored and observed to FAIL (Red) before the
implementation tasks that make them pass (Green). The foundational types in Phase 2
may need to compile first so test files reference real names; if so, stub those
types with `todo!()`/`unimplemented!()` bodies so the tests compile and fail rather
than pass prematurely.

**Organization**: Tasks are grouped by user story to enable independent
implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files or non-overlapping regions, no
  dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

Single Rust library crate inside the existing Cargo workspace. All work lives in:

- `crates/webe_args/Cargo.toml` — crate manifest
- `crates/webe_args/src/lib.rs` — public API + `#[cfg(test)]` unit tests
- `crates/webe_args/README.md` — usage documentation
- `crates/webe_args/tests/cli.rs` — default-harness integration tests
- `src/lib.rs` — workspace facade re-export (`pub use webe_args as args`)

> **Single-file note**: Most source tasks edit `crates/webe_args/src/lib.rs`. Tasks
> touching the same file are intentionally **not** marked `[P]` and must run
> sequentially to avoid edit conflicts. `[P]` is reserved for tasks in different
> files (e.g., `Cargo.toml`, `README.md`, `tests/cli.rs`).

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare the crate manifest and project wiring for the revamp.

- [X] T001 Update `crates/webe_args/Cargo.toml`: remove the `manual-cli-test`
  feature, the `[[test]]` block with `harness = false`, and `required-features`, so
  `cargo test -p webe_args` runs under the default harness (FR-016).
- [X] T002 [P] Confirm the workspace facade re-export `pub use webe_args as args`
  remains present and feature-gated in `src/lib.rs` (C15); leave it intact.
- [X] T003 [P] In `crates/webe_args/src/lib.rs`, add the crate-level lint guards
  required by the constitution (e.g. `#![forbid(unsafe_code)]` and
  `#![warn(missing_docs)]`) so quality gates are enforced from the start (NF2).

**Checkpoint**: Crate builds with default harness; facade and lint gates in place.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types and the internal token scanner that ALL user stories depend
on. Replaces the existing `env::args()`-based, `panic!`-driven implementation with
deterministic, caller-supplied-token logic.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 In `crates/webe_args/src/lib.rs`, define the option-definition type
  (long name, optional short alias, description, `is_required`, `is_flag`, optional
  validation predicate) per data-model "Option Definition", replacing/superseding
  the old `ArgOpts`/`DEFAULT_OPTS`.
- [X] T005 In `crates/webe_args/src/lib.rs`, define the `ParseFailure` type with all
  documented variants: `MissingRequired`, `MissingValue`, `InvalidValue`,
  `UnknownOption`, `DuplicateOption`, `UnexpectedArgument`, `ConflictingDefinition`,
  `UndeclaredLookup`, each carrying the option name or offending token; implement
  `Debug`, `Display`, and `std::error::Error` (data-model "Parse Failure").
- [X] T006 In `crates/webe_args/src/lib.rs`, define the option-result type
  (`Value(String)` / `Flag` / `Absent` / failure) per data-model "Option Result".
- [X] T007 In `crates/webe_args/src/lib.rs`, define the `ValidationReport` type
  (success or ordered `failures: Vec<ParseFailure>`) per data-model "Validation
  Report".
- [X] T008 In `crates/webe_args/src/lib.rs`, define the registry type (replacing
  `Args`) holding option definitions, with a constructor and an `add` method that
  preserves registration order for deterministic reporting.
- [X] T009 In `crates/webe_args/src/lib.rs`, implement the internal single-pass
  token scanner over a caller-supplied `&[String]` slice: recognizes
  `--long`/`-short` option tokens, treats any `-`-prefixed token as an option token
  (never a value), and binds the next non-option token as a value (token
  interpretation rules in data-model "Command-Line Input"). No allocation-heavy or
  O(n²) scans (NF1).

**Checkpoint**: Core types and scanner compile; the crate exposes the data shapes
every story builds on.

---

## Phase 3: User Story 1 - Define and Read Simple Options (Priority: P1) 🎯 MVP

**Goal**: Developers can define value options, flag options, optional/required
options, then read the correct value or flag state from supplied tokens without
custom parsing.

**Independent Test**: Define a required value option, an optional value option, and
a boolean flag, run the reader against representative tokens, and confirm each
result matches the input (quickstart scenarios 1–5).

### Tests for User Story 1 (write FIRST, observe FAIL before implementing) ⚠️

- [X] T010 [P] [US1] In `crates/webe_args/src/lib.rs` under `#[cfg(test)]`, add unit
  tests for value read, flag read, optional-absent, and verbatim value preservation
  (quickstart scenarios 1, 2, 3, 5; C2/C3/C4/NF3). Confirm they FAIL (Red).
- [X] T011 [P] [US1] In `crates/webe_args/tests/cli.rs`, add default-harness
  integration tests covering the US1 read paths over caller-supplied tokens. Confirm
  they FAIL (Red).

### Implementation for User Story 1 (make T010–T011 pass)

- [X] T012 [US1] In `crates/webe_args/src/lib.rs`, implement option-definition
  registration helpers/builders so a developer can declare value options, flags,
  and required/optional status via the registry `add` path (C1, FR-001).
- [X] T013 [US1] In `crates/webe_args/src/lib.rs`, implement reading a declared
  value option from supplied tokens, returning the value exactly as provided (C2,
  FR-002, NF3).
- [X] T014 [US1] In `crates/webe_args/src/lib.rs`, implement reading a declared flag
  option as a present-flag result that does NOT consume a following unrelated token
  as its value (C3, FR-003).
- [X] T015 [US1] In `crates/webe_args/src/lib.rs`, implement the absent-optional
  path so an omitted optional option returns a successful no-value result (C4,
  FR-004). Re-run tests → GREEN.

**Checkpoint**: User Story 1 is fully functional and independently testable — the
MVP read path works without the process environment.

---

## Phase 4: User Story 2 - Diagnose Missing or Invalid Input (Priority: P2)

**Goal**: Full startup validation distinguishes missing required options, missing
values, invalid values, duplicates, unknown options, unexpected positionals, and
undeclared lookups, reporting all failures in deterministic order.

**Independent Test**: Define options with required/validation rules, pass
incomplete or invalid tokens, and confirm each failure reports the correct reason
and option/token, with multi-failure input ordered deterministically (quickstart
scenarios 6–14, 16, 17).

### Tests for User Story 2 (write FIRST, observe FAIL before implementing) ⚠️

- [X] T016 [P] [US2] In `crates/webe_args/src/lib.rs` under `#[cfg(test)]`, add unit
  tests for each failure kind: missing-required, missing-value (end + dash next),
  invalid value, unknown option, duplicate (repeated long, long+short), unexpected
  positional, undeclared lookup (quickstart scenarios 6–9, 11–14, 16). Confirm they
  FAIL (Red).
- [X] T017 [P] [US2] In `crates/webe_args/tests/cli.rs`, add an integration test for
  the all-failures, deterministic-order report using mixed bad input such as
  `--bogus extra --port` (quickstart scenario 17; C14). Confirm it FAILS (Red).

### Implementation for User Story 2 (make T016–T017 pass)

- [X] T018 [US2] In `crates/webe_args/src/lib.rs`, implement full command-line
  validation that walks all declared options and all supplied tokens in one pass and
  produces a `ValidationReport` (C10, FR-010).
- [X] T019 [US2] In `crates/webe_args/src/lib.rs`, emit `MissingRequired` for
  omitted required options (C5, FR-005) and `MissingValue` for value options at end
  of input or immediately before a `-`-prefixed token (C6, FR-006).
- [X] T020 [US2] In `crates/webe_args/src/lib.rs`, apply the optional validation
  predicate and emit `InvalidValue` naming the option when the rule rejects a value;
  a passing value succeeds (C7, FR-007).
- [X] T021 [US2] In `crates/webe_args/src/lib.rs`, emit `UnknownOption` for option
  tokens with no declared long name/alias (C10, FR-010) and `UnexpectedArgument` for
  bare positional tokens not consumed as values (C12, FR-012).
- [X] T022 [US2] In `crates/webe_args/src/lib.rs`, emit `DuplicateOption` for a
  repeated option (repeated long, repeated short, or long+short for the same option)
  (C11, FR-011).
- [X] T023 [US2] In `crates/webe_args/src/lib.rs`, return a distinct
  `UndeclaredLookup` programming-error result when reading an option name that was
  never defined (C8, FR-008).
- [X] T024 [US2] In `crates/webe_args/src/lib.rs`, implement deterministic failure
  ordering in `ValidationReport`: conflicting definitions first, then input failures
  by left-to-right token position, then missing-required by registration order (C14,
  FR-010 clarification, SC-003). Re-run tests → GREEN.

**Checkpoint**: User Stories 1 AND 2 both work independently; startup validation is
actionable and complete.

---

## Phase 5: User Story 3 - Use Long and Short Option Forms (Priority: P3)

**Goal**: An option may declare a long name and an optional short alias, and either
form produces the same result.

**Independent Test**: Define an option with both forms and confirm long and short
inputs yield identical parsed results (quickstart scenario 4).

### Tests for User Story 3 (write FIRST, observe FAIL before implementing) ⚠️

- [X] T025 [P] [US3] In `crates/webe_args/src/lib.rs` under `#[cfg(test)]`, add unit
  tests proving long-form and short-form inputs produce identical results, and that
  a duplicate long name / short alias yields `ConflictingDefinition` (quickstart
  scenarios 4, 15; C9/C13). Confirm they FAIL (Red).

### Implementation for User Story 3 (make T025 pass)

- [X] T026 [US3] In `crates/webe_args/src/lib.rs`, ensure the scanner and read/
  validate paths resolve a short alias to the same option as its long name, yielding
  identical results for either form (C9, FR-009).
- [X] T027 [US3] In `crates/webe_args/src/lib.rs`, enforce definition-time conflict
  detection for duplicate long names and duplicate short aliases, emitting
  `ConflictingDefinition` before input is accepted as valid (C13, FR-013). Re-run
  tests → GREEN.

**Checkpoint**: All three behavior stories are independently functional and tested.

---

## Phase 6: User Story 4 - Learn and Verify Behavior (Priority: P4)

**Goal**: Documentation explains every supported behavior with runnable examples,
and the full behavior set is protected by tests runnable via the normal flow.

**Independent Test**: Follow the README/rustdoc examples from a clean project and
run `cargo test -p webe_args` (incl. doctests) with no manual command-line text
(quickstart "Documentation check" and "Run the crate's tests").

> **Test-first note**: US4 adds documentation and verification only — no new runtime
> behavior — so the Red→Green ordering of Constitution II does not apply here. The
> doctests authored in T028 are the documentation itself; T030 simply runs them.

### Implementation for User Story 4

- [X] T028 [P] [US4] In `crates/webe_args/src/lib.rs`, add crate-level rustdoc and
  per-item doc comments (intent, params, failure modes) with runnable doctests for
  value options, flags, optional/required options, aliases, validation, duplicate
  failures, dash-prefixed token behavior, unexpected-positional behavior, and
  all-failures validation (FR-014, NF2, SC-006).
- [X] T029 [P] [US4] Rewrite `crates/webe_args/README.md` with usage examples for
  every supported option shape and failure mode, and document the breaking API
  changes from the previous `Args`/`parse_args` design (FR-014, SC-004, Constitution
  III).

### Tests for User Story 4

- [X] T030 [US4] Ensure `cargo test -p webe_args --doc` passes so documentation
  examples compile and stay in sync with behavior (FR-016, SC-004).
- [X] T031 [P] [US4] In `crates/webe_args/tests/cli.rs`, add a traceability pass
  confirming every quickstart scenario (1–17) maps to at least one passing test
  (FR-015, SC-002).

**Checkpoint**: Documentation and tests fully cover the supported simple-option
scope with no undocumented public behavior.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Quality gates and final validation across all stories.

- [X] T032 Run `cargo fmt --check` and fix any formatting diffs across
  `crates/webe_args/` (NF2).
- [X] T033 Run `cargo clippy -p webe_args -- -D warnings` and resolve all warnings;
  confirm no `panic!`/`unwrap`/`expect`/`unsafe` remain in library paths (NF2).
- [X] T034 [P] Sanity-check linear performance for ~20 declared options and ~100
  tokens (no O(n²) scans) per NF1/SC-005; adjust the scanner if needed.
- [X] T035 Run `cargo test --workspace` and `cargo build` (default features) to
  confirm all tests pass and `webe::args` resolves via the facade (C15, FR-016).
- [X] T036 Execute the `quickstart.md` validation steps end-to-end and confirm
  scenarios 1–17 and all success signals pass.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories.
- **User Stories (Phase 3–6)**: All depend on Foundational completion.
  - US1 (P1) is the MVP and should be completed first.
  - US2 (P2) builds on the scanner and types but is independently testable.
  - US3 (P3) refines alias/conflict handling and is independently testable.
  - US4 (P4) documents and traces behaviors delivered by US1–US3.
- **Polish (Phase 7)**: Depends on all desired user stories being complete.

### User Story Dependencies

- **US1 (P1)**: Depends only on Foundational. No dependency on other stories.
- **US2 (P2)**: Depends on Foundational; reuses US1's scanner/types but adds its own
  validation paths — independently testable.
- **US3 (P3)**: Depends on Foundational; alias resolution touches paths shared with
  US1/US2 but is independently testable.
- **US4 (P4)**: Documents behaviors from US1–US3; its doctests assume those paths
  exist.

### Within Each User Story

- Test tasks come FIRST and MUST fail (Red) before the implementation tasks that
  make them pass (Green), per Constitution II.
- Source implementation tasks in `src/lib.rs` are sequential (same file).
- The two test tasks per story (`src/lib.rs` `#[cfg(test)]` and `tests/cli.rs`) are
  `[P]` relative to each other only where they touch different files.
- Story complete before moving to next priority.

### Parallel Opportunities

- Setup: T002 and T003 can run in parallel with each other (different files).
- Per-story test pairs marked `[P]` (e.g., T010+T011, T016+T017) can be authored
  together since they touch different files; both must be Red before implementation.
- Same-file `src/lib.rs` implementation tasks must run sequentially.

---

## Parallel Example: User Story 1

```bash
# Author the failing tests first (Red), in parallel across the two files:
Task: "T010 unit tests in crates/webe_args/src/lib.rs #[cfg(test)]"
Task: "T011 integration tests in crates/webe_args/tests/cli.rs"
# Then implement T012–T015 in src/lib.rs (sequential) until tests pass (Green).
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories).
3. Complete Phase 3: User Story 1.
4. **STOP and VALIDATE**: Run `cargo test -p webe_args`; confirm read paths work.
5. Demo the read API as the MVP.

### Incremental Delivery

1. Setup + Foundational → core types & scanner ready.
2. US1 → read paths → test → MVP.
3. US2 → full validation & diagnostics → test.
4. US3 → long/short equivalence & conflict detection → test.
5. US4 → docs, doctests, traceability → test.
6. Polish → fmt/clippy/perf/workspace validation.

---

## Notes

- `[P]` tasks = different files, no dependencies.
- Same-file `src/lib.rs` tasks are sequential by design.
- `[Story]` label maps each task to its user story for traceability.
- No new dependencies — the crate stays `std`-only.
- Replace, do not extend, the old `env::args()`/`panic!` implementation.
- Commit after each task or logical group.
