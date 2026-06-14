# Tasks: Revamp Webe Web

**Input**: Design documents from `/specs/002-revamp-webe-web/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/public-api.md](contracts/public-api.md), [quickstart.md](quickstart.md)

**Tests**: INCLUDED. The Webe Toolkit constitution makes test-first development
NON-NEGOTIABLE (Principle II) and FR-017 mandates automated coverage of every
supported behavior. Each user story writes its tests before implementation (Red →
Green).

**Organization**: Tasks are grouped by user story (P1–P4) so each can be
implemented and verified independently. The reorganization in Phase 2 is a shared
prerequisite for the behavior changes in the story phases.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: US1–US4; Setup/Foundational/Polish phases have no story label
- All paths are relative to the repository root

## Path Conventions

Single async library crate at `crates/webe_web/`, surfaced via the `webe` facade at
`src/lib.rs`. Unit tests live beside source under `#[cfg(test)]`; integration tests
live in `crates/webe_web/tests/`; the facade smoke test lives in `tests/http.rs`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish the new module skeleton and scaffolding referenced by every
later phase, without yet changing behavior.

- [X] T001 Create empty module files `crates/webe_web/src/route.rs`, `crates/webe_web/src/error.rs`, and `crates/webe_web/src/body.rs`, and declare them in `crates/webe_web/src/lib.rs` (keep `pub mod processor;` and existing modules)
- [X] T002 [P] Create empty chunked encoder module `crates/webe_web/src/encoding/chunked_encoder.rs` and declare it in `crates/webe_web/src/encoding/mod.rs`
- [X] T003 [P] Create empty integration test files `crates/webe_web/tests/routing.rs`, `crates/webe_web/tests/request_parsing.rs`, `crates/webe_web/tests/body.rs`, `crates/webe_web/tests/responses.rs`, and `crates/webe_web/tests/responders.rs` each with a `// placeholder` so the crate compiles
- [X] T004 [P] Verify baseline gates run: `cargo fmt --check`, `cargo clippy -p webe_web -- -D warnings`, `cargo build -p webe_web` (record current clippy warnings to clear during the revamp)

**Checkpoint**: Crate still builds; new empty modules and test files exist.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Perform the module reorganization and introduce the typed error and
body-framing foundations. These are blocking prerequisites for all user stories
because every story phase depends on the new module boundaries and `WebError`.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T005 Define the consolidated `WebError` taxonomy in `crates/webe_web/src/error.rs` with categories `Bind`, `Accept`, `Request`, `Version`, `Body`, `Routing`, `Response`, `Responder`, deriving `Debug` and implementing `Display` so each variant names what failed (FR-014, data-model.md "Failure")
- [X] T006 Add `From<RequestError>`, `From<ResponseError>`, and `From<ServerError>` conversions into `WebError` in `crates/webe_web/src/error.rs` so existing call sites gain a categorized surface (research.md Decision 7)
- [X] T007 Extract `Route`, `RouteMap`, `find_best_route`, and `parse_route_params` from `crates/webe_web/src/server.rs` into `crates/webe_web/src/route.rs`; re-export `Route` and `RouteMap` from `server` for source compatibility (plan.md Structure Decision)
- [X] T008 Slim `crates/webe_web/src/server.rs` to the `Server` lifecycle only (`new` bind, `start` accept/spawn), returning `WebError` instead of `ServerError` at the public boundary (FR-001)
- [X] T009 Repurpose `crates/webe_web/src/processor.rs`: move the former `process_stream` per-connection request loop here as the connection lifecycle entry point; delete the dead `ProcessError` enum (research.md Decision 6)
- [X] T010 Move request/response body-framing decision logic into `crates/webe_web/src/body.rs` (choose request body reader from `Content-Length`/`chunked`; choose response framing), leaving `processor.rs` to call into it (FR-009, data-model.md "Body Framing")
- [X] T011 Remove the stray `dbg!(&parse_result);` from `crates/webe_web/src/request.rs` and confirm header parsing still lowercases names and comma-combines duplicates (FR-008, Constitution I)
- [X] T012 Confirm the reorganized crate compiles and the existing `basic_server` example still builds against any moved public paths: `cargo build -p webe_web -p basic_server`

**Checkpoint**: Crate is reorganized (route/server/processor/error/body) and builds;
no behavior changes yet beyond `dbg!` removal. User stories can now begin.

---

## Phase 3: User Story 1 - Serve Basic HTTP Requests Reliably (Priority: P1) 🎯 MVP

**Goal**: Dependable request-to-response path: supported `HTTP/1.1` requests reach
their responder; not-found, method-not-allowed, and bad-request failures return
documented statuses without stopping the server.

**Independent Test**: Start a server with a static route, send supported requests
over a local connection, and confirm status/headers/body/connection match the docs
for success, `404`, `405`, and `400`.

### Tests for User Story 1 (write first; must FAIL before implementation) ⚠️

- [X] T013 [P] [US1] In `crates/webe_web/tests/request_parsing.rs`, add tests: valid `HTTP/1.1` request line accepted; malformed request line → `400`; missing header separator → `400`; request-line over `MAX_REQUEST_LINE_SIZE` → `400`; header block over `MAX_HEADERS_SIZE` → `400`; non-`HTTP/1.1` version → `505` (FR-006, FR-007, research.md Decision 3)
- [X] T014 [P] [US1] In `crates/webe_web/tests/routing.rs`, add tests: registered route returns responder output; unregistered path → `404`; path matches a registered pattern but method does not → `405`; server remains available after each failure (FR-002, research.md Decision 2)
- [X] T015 [P] [US1] In `crates/webe_web/tests/request_parsing.rs` (or a new `crates/webe_web/tests/server.rs`), add a bind-failure test: `Server::new` on an already-bound address/port returns a typed `WebError::Bind` (no panic) (FR-001, FR-017)

### Implementation for User Story 1

- [X] T016 [US1] Enforce `HTTP/1.1`-only in `crates/webe_web/src/request.rs`: validate the parsed version token and surface a `Version` failure mapped to `505` (FR-006, FR-007)
- [X] T017 [US1] Enforce the header-block size limit in `crates/webe_web/src/request.rs`: when the header section exceeds `MAX_HEADERS_SIZE`, return a typed `Request` failure mapped to `400` (replaces the existing `TODO: handle max header size error`) (FR-006, spec Edge Cases)
- [X] T018 [US1] In `crates/webe_web/src/route.rs`, separate path-pattern matching from method matching so a path match with no method match yields a `405` and no path match yields `404` (FR-002, FR-004, data-model.md "Route Map")
- [X] T019 [US1] In `crates/webe_web/src/processor.rs`, map request-parse, version, routing, and responder failures to the documented static error responses (`400`/`404`/`405`/`505`) while keeping the connection loop alive for subsequent clients (FR-007, FR-014, US1 scenarios 2–4)
- [X] T020 [US1] Add `#[cfg(test)]` unit tests in `crates/webe_web/src/route.rs` for `find_best_route` returning the 404 vs 405 distinction on a small route table
- [X] T021 [US1] Run `cargo test -p webe_web --test routing --test request_parsing`; confirm Red→Green and all US1 tests pass

**Checkpoint**: US1 is independently testable — success, `404`, `405`, and `400`
paths verified; server stays up across failures. This is the MVP.

---

## Phase 4: User Story 2 - Build Predictable Route Handlers (Priority: P2)

**Goal**: Deterministic exact, parameterized, and terminal route selection with
captured parameters delivered to responders exactly as documented.

**Independent Test**: Register exact + parameterized routes with distinguishable
responders, send requests exercising each matching case, and confirm the chosen
responder and captured parameters are deterministic.

### Tests for User Story 2 (write first; must FAIL before implementation) ⚠️

- [X] T022 [P] [US2] In `crates/webe_web/tests/routing.rs`, add tests: exact route wins over a parameterized route on the same path; most-specific parameterized route wins on ties (earliest wildcard); leading-slash and no-leading-slash registrations match the same path (FR-003, FR-004)
- [X] T023 [P] [US2] In `crates/webe_web/tests/routing.rs`, add tests: non-terminal `<param>` captures one segment; terminal `<param>` captures the remaining path; captured name/value reach the responder exactly (FR-005)
- [X] T024 [P] [US2] In `crates/webe_web/tests/routing.rs`, add a determinism test registering ≥25 mixed exact/parameterized routes and asserting stable selection (SC-006)

### Implementation for User Story 2

- [X] T025 [US2] In `crates/webe_web/src/route.rs`, confirm/adjust `find_best_route` selection ordering (exact > most-matching-parts > earliest-wildcard) is deterministic and documented with doc comments (FR-004)
- [X] T026 [US2] In `crates/webe_web/src/route.rs`, confirm `parse_route_params` returns documented name/value pairs for non-terminal and terminal captures; add doc comments (FR-005, data-model.md "Route Parameter")
- [X] T027 [US2] Ensure `RouteMap::add_route` path normalization (leading slash) is applied consistently and documented (FR-003)
- [X] T028 [US2] Run `cargo test -p webe_web --test routing`; confirm Red→Green and all US2 tests pass

**Checkpoint**: US2 verified independently — routing selection and parameter capture
are deterministic across ≥25 routes.

---

## Phase 5: User Story 3 - Handle Request Bodies and Responses Safely (Priority: P3)

**Goal**: Accept supported request body framing and write static, file-backed, or
custom responses with correct framing and connection handling, without unbounded
memory use.

**Independent Test**: Send requests with supported body framing and return responses
with/without bodies; confirm size limits, body delivery, and keep-alive/close follow
the docs.

### Tests for User Story 3 (write first; must FAIL before implementation) ⚠️

- [X] T029 [P] [US3] In `crates/webe_web/tests/body.rs`, add tests: `Content-Length` body read exactly; final-`chunked` body decoded correctly; both headers present → `400`; unparseable `Content-Length` → `400`; unsupported transfer coding rejected; responder not invoked on rejection (FR-007, FR-009, US3-2)
- [X] T030 [P] [US3] In `crates/webe_web/tests/responses.rs`, add tests: response with known-length body sends `Content-Length`; streamed unknown-length body sends `Transfer-Encoding: chunked`; bodyless response sends neither; client receives complete body + headers (FR-010, research.md Decision 1)
- [X] T031 [P] [US3] In `crates/webe_web/tests/responses.rs`, add tests: two sequential keep-alive requests on one connection both respond correctly; `Connection: close` request closes after responding (SC-004)

### Implementation for User Story 3

- [X] T032 [US3] In `crates/webe_web/src/body.rs`, implement the request-framing rules: reject both-headers-present (`400`), parse single `Content-Length`, accept final-`chunked`, reject other codings; except framing headers from comma-combining (FR-007, FR-008, FR-009)
- [X] T033 [US3] Implement the chunked response encoder in `crates/webe_web/src/encoding/chunked_encoder.rs` (streams a body as chunked transfer-coding) with `#[cfg(test)]` unit tests (FR-009, SC-005)
- [X] T034 [US3] In `crates/webe_web/src/response.rs`, frame outgoing bodies: set `Content-Length` when length is known, use the chunked encoder when streaming unknown length, send neither when bodyless; stream rather than buffer the whole body (FR-010, SC-005)
- [X] T035 [US3] In `crates/webe_web/src/response.rs` / `crates/webe_web/src/processor.rs`, reconcile `keep_alive` against framing and the request `Connection` header so a connection is only reused when the body is self-delimiting (FR-010, SC-004)
- [X] T036 [US3] Surface a typed `Response` failure when a body reader fails mid-write instead of a generic internal error (FR-014, spec Edge Cases)
- [X] T037 [US3] Run `cargo test -p webe_web --test body --test responses`; confirm Red→Green and all US3 tests pass

**Checkpoint**: US3 verified independently — supported body framing, response
framing, streaming, and keep-alive/close behavior all correct.

---

## Phase 6: User Story 4 - Diagnose and Maintain the Web Crate (Priority: P4)

**Goal**: Documentation makes the supported scope obvious, failures are actionable,
the example reflects reality, and the documented test suite runs from a clean
checkout.

**Independent Test**: Follow the published example and run the documented test suite
from a clean checkout without manual request crafting beyond automated tests.

### Tests for User Story 4 (write first; must FAIL before implementation) ⚠️

- [X] T038 [P] [US4] In `crates/webe_web/tests/responders.rs`, add tests covering each built-in responder's success path and relevant failure path: `StaticResponder`, `OptionsResponder` (preflight), `SpaResponder` (fallback + missing index), `FileResponder` (FR-012)
- [X] T039 [P] [US4] In `crates/webe_web/tests/responders.rs`, add `FileResponder` safety tests: path traversal outside mount → denied; directory path; missing file; symlink target; write attempt — each with a deterministic status (FR-013, spec Edge Cases)
- [X] T040 [P] [US4] Rewrite `tests/http.rs` as an async facade-level smoke test using `webe::web` (`Server::new` + `RouteMap` + `start`), removing the `#[ignore]` and stale synchronous API references (FR-018, SC-007)

### Implementation for User Story 4

- [X] T041 [P] [US4] Add doc comments to all public items (Server, Route, RouteMap, Request, Response, Responder, built-in responders, `WebError`, Status, Validation) describing intent, params, and failure modes (FR-014, Constitution I)
- [X] T042 [US4] Create `crates/webe_web/README.md` documenting the supported HTTP/1.1 subset, explicit exclusions, a usage example, and migration notes for the breaking changes (error consolidation, response framing, `Route`/`RouteMap` module move) (FR-015, FR-020, SC-008)
- [X] T043 [US4] Update `examples/basic_server/src/main.rs` only as needed for moved public module paths; confirm it demonstrates the documented happy path (FR-018, SC-001)
- [X] T044 [US4] Ensure every `WebError` category's `Display` names the failure category and resolution hint where possible (FR-014, SC-003)
- [X] T045 [US4] Run `cargo test -p webe_web --test responders` and `cargo test --workspace`; confirm all tests pass with no ignored supported-scope tests

**Checkpoint**: US4 verified — built-in responders covered, file-serving safety
enforced, docs/README/example accurate, facade test runs.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final validation across all stories and constitution gates.

- [X] T046 [P] Run `cargo fmt --check` and `cargo clippy --workspace -- -D warnings`; clear any remaining warnings (Constitution I)
- [X] T047 Run the full `cargo test --workspace` and walk the manual scenarios in [quickstart.md](quickstart.md) (US1–US3 curl/nc checks) to confirm documented behavior
- [X] T048 [P] Audit library paths for `.unwrap()`/`.expect()` and replace with typed `WebError` propagation (Constitution I)
- [X] T049 Verify scope completeness: every documented supported behavior maps to ≥1 test (SC-002) and no undocumented public behavior remains in scope (SC-008)

---

## Dependencies & Execution Order

```text
Phase 1 (Setup) ─────────────► Phase 2 (Foundational, BLOCKING)
                                       │
        ┌──────────────┬───────────────┼───────────────┐
        ▼              ▼               ▼               ▼
   Phase 3 (US1)   Phase 4 (US2)   Phase 5 (US3)   Phase 6 (US4)
        │              │               │               │
        └──────────────┴───────────────┴───────────────┘
                                       │
                                       ▼
                              Phase 7 (Polish)
```

- **Setup → Foundational**: T001 before T002–T004 (lib.rs declarations); Foundational
  depends on Setup.
- **Foundational is blocking**: T005–T012 must complete before any story phase. Within
  it: T005 → T006; T007 (extract route) → T008/T009; T009 → T010.
- **Story independence**: US1–US4 each depend only on Foundational, not on each other,
  so they can proceed in parallel by different contributors. They share
  `crates/webe_web/tests/routing.rs` (US1+US2) and `responses.rs` (US3) — coordinate
  edits to shared test files.
- **Within a story**: tests (write-first) before implementation; the story's final
  `cargo test` task verifies Green.
- **Polish** runs after all stories.

## Parallel Execution Examples

- **Setup**: T002, T003, T004 can run together after T001.
- **US1 tests**: T013, T014, and T015 touch different files/concerns → parallel.
- **US3 tests**: T029 (`body.rs`), T030 + T031 (`responses.rs`) — T030/T031 share a
  file; T029 is parallel to them.
- **US4**: T038, T039, T040 (different files) and doc task T041 can run in parallel.
- **Cross-story**: once Foundational is done, one contributor can take US1 while
  another takes US3, since they touch largely different modules (`request`/`route`
  vs `body`/`response`/`encoding`).

## Implementation Strategy

- **MVP first**: Complete Phase 1 → Phase 2 → Phase 3 (US1). This delivers a reliable
  request-to-response path with correct `404`/`405`/`400`/`505` handling — the core
  value of the crate.
- **Incremental delivery**: Add US2 (routing determinism), then US3 (body/response
  framing + keep-alive), then US4 (docs, built-in responder coverage, file-serving
  safety, facade test).
- **Each story is a checkpoint**: stop after any story and the crate is in a
  consistent, tested state for the behaviors delivered so far.
