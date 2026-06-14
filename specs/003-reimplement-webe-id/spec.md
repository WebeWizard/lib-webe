# Feature Specification: Reimplement Webe ID

**Feature Branch**: `003-reimplement-webe-id`

**Created**: 2026-06-14

**Status**: Draft

**Input**: User description: "Create a new spec for re-implementing WebeID / webe_id in this repo. The original repo is found here: `https://github.com/WebeWizard/WebeID`

I don't care how you implement it.  I really care about performance and the concepts that make up a WebeID.  Consider making it Tokio friendly"

## Clarifications

### Session 2026-06-14

- Q: What should happen when one node exhausts the 65,536 WebeID sequence values available within one millisecond? → A: Fail-fast generation by default; provide separate documented bounded backpressure for async/server-style callers.
- Q: What performance gate should the spec require for normal single-generator WebeID generation? → A: Do not run comparison benchmarks against the original WebeID; require self-contained performance reporting.
- Q: For clock rewinds, should a generator recover automatically after real time catches up, or remain unusable until rebuilt? → A: Temporarily fail while time is behind the last generated duration, then resume after time catches up.
- Q: What self-contained performance target should the spec require for normal single-generator generation? → A: Benchmark reporting only, with no pass/fail performance gate.
- Q: For restart safety, what should the persisted restart marker represent? → A: Persist the full last generated WebeID and derive restart safety from its components.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Generate WebeIDs With Preserved Semantics (Priority: P1)

A developer can generate compact WebeIDs that preserve the original concept: a 64-bit value composed from custom-epoch time, node identity, and a per-time-slice sequence number.

**Why this priority**: The value of this feature is the WebeID concept itself. If the generated IDs do not preserve the compact layout, uniqueness model, and sortable time behavior, the reimplementation has missed the core purpose.

**Independent Test**: Can be tested by configuring a generator with a known epoch and node, generating IDs across one or more milliseconds, and confirming each ID is unique, sortable by generation time, and decomposes into the expected time, node, and sequence components.

**Acceptance Scenarios**:

1. **Given** a valid custom epoch and node identifier, **When** a developer generates multiple WebeIDs in the same millisecond, **Then** each ID is unique and the sequence component advances without changing the time or node components.
2. **Given** a generator produces IDs over later milliseconds, **When** those IDs are compared numerically, **Then** IDs from later milliseconds sort after IDs from earlier milliseconds.
3. **Given** two generators use different node identifiers within the same time interval, **When** each generator produces an ID with the same sequence position, **Then** the IDs differ by their node component.
4. **Given** a generated WebeID, **When** a developer decomposes it, **Then** the result identifies the time duration since the custom epoch, node identifier, and sequence value used to create it.

---

### User Story 2 - Protect Uniqueness During Restarts and Clock Problems (Priority: P2)

A developer can use WebeIDs safely across process restarts, clock drift, and capacity boundaries without silent duplicate generation.

**Why this priority**: WebeIDs are only useful if duplicate prevention is reliable under the failure modes that commonly affect time-based ID generators.

**Independent Test**: Can be tested by creating generators with valid and invalid restart markers, simulating backward time movement, and forcing sequence capacity exhaustion, then confirming each risk produces a documented outcome instead of a duplicate ID.

**Acceptance Scenarios**:

1. **Given** the full last generated WebeID from a previous run has been persisted, **When** a new generator starts after time has advanced beyond that ID's time component, **Then** generation can continue without reusing earlier WebeIDs.
2. **Given** a persisted last generated WebeID whose time component is equal to or ahead of the current time duration, **When** a generator is created, **Then** creation fails with a documented restart-safety outcome.
3. **Given** the observed clock moves backward after generation has begun, **When** the developer requests another ID before time catches up to the last generated duration, **Then** the generator refuses to create a potentially duplicate ID and reports a documented clock-rewind outcome.
4. **Given** a generator has reported a clock-rewind outcome, **When** the observed clock later reaches or passes the last generated duration, **Then** generation can resume without rebuilding the generator or reusing an earlier WebeID.
5. **Given** all sequence values for a node within the same millisecond have been consumed, **When** another default generation request arrives before time advances, **Then** the generator fails fast with a documented capacity outcome and does not emit a duplicate.

---

### User Story 3 - Meet Server-Workload Performance Expectations (Priority: P3)

A developer building request-heavy services can generate WebeIDs at high volume with bounded resource use and behavior that is friendly to asynchronous server workloads.

**Why this priority**: The user explicitly values performance. WebeID generation is likely to sit on hot request paths, so its behavior must be measurable, predictable, and safe under concurrency.

**Independent Test**: Can be tested with repeatable benchmarks and concurrent generation tests that measure throughput, latency, duplicate rate, resource growth, and behavior when per-millisecond capacity is reached.

**Acceptance Scenarios**:

1. **Given** a single active generator under normal load, **When** a benchmark requests a large number of IDs, **Then** throughput and latency are reported with enough context to compare future changes without duplicate IDs.
2. **Given** many concurrent request handlers share the ID generation facility as documented, **When** they request IDs simultaneously, **Then** all successful IDs are unique and caller-visible delays remain within the documented bounds.
3. **Given** generation demand exceeds the per-node per-millisecond capacity, **When** callers use the documented server-style backpressure behavior, **Then** the system waits only within the documented bound and never falls back to unbounded spinning, memory growth, or silent duplicate generation.
4. **Given** an asynchronous service uses WebeID generation in a request path, **When** IDs are requested under normal load, **Then** generation does not require blocking waits on the service's async execution path.

---

### User Story 4 - Understand and Validate the WebeID Model (Priority: P4)

A developer or maintainer can understand the WebeID layout, uniqueness domain, boundaries, errors, and performance profile from documentation and tests.

**Why this priority**: The reimplementation should make the original concept easier to trust and maintain in this workspace, not merely recreate an opaque generator.

**Independent Test**: Can be tested by following the documentation from a clean checkout, generating and decoding sample IDs, reviewing boundary examples, and running the documented verification suite.

**Acceptance Scenarios**:

1. **Given** a developer new to WebeID, **When** they read the documentation, **Then** they can identify the 5-byte time component, 1-byte node component, 2-byte sequence component, custom epoch, uniqueness domain, and capacity limits without reading source code.
2. **Given** a maintainer changes generation, parsing, or error behavior, **When** they run the documented tests, **Then** success paths, boundary paths, and failure paths for every documented behavior are covered.
3. **Given** behavior differs intentionally from the original WebeID crate, **When** the release-facing documentation is reviewed, **Then** the difference and migration impact are clearly documented.

### Edge Cases

- The custom epoch is later than the current clock reading.
- The current clock reading is beyond the maximum representable time range for a 5-byte millisecond duration.
- The node identifier is at the lower or upper valid boundary.
- A caller attempts to represent or configure a node identifier outside the valid node range.
- The observed clock moves backward after one or more IDs have been generated, then later catches up to the last generated duration.
- A persisted restart WebeID is equal to, greater than, malformed relative to, or incompatible with the current custom epoch and node expectations.
- More than 65,536 IDs are requested for the same node within one millisecond.
- Multiple live generators are configured with the same epoch and node identifier.
- IDs are generated concurrently through the documented sharing pattern.
- A developer decomposes the minimum possible ID, the maximum possible ID, and IDs from arbitrary external input.
- Numeric, byte, decimal, and hexadecimal representations must preserve the same conceptual components.
- Performance benchmarks run on modest hardware with enough volume to expose allocation, locking, or wait behavior, without requiring comparison to the original WebeID crate.
- Default generation and documented server-style backpressure requests are made while the generator is at or near per-millisecond capacity.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The feature MUST define a WebeID as a 64-bit value whose canonical components are 5 bytes of milliseconds since a custom epoch, 1 byte of node identifier, and 2 bytes of sequence number. Acceptance: documentation and decomposition tests agree on the exact component sizes and their order.
- **FR-002**: WebeIDs MUST preserve numeric ordering by time component before node and sequence components. Acceptance: IDs generated over increasing millisecond durations sort in the same order as those durations.
- **FR-003**: The feature MUST allow developers to configure a custom epoch and a node identifier within the valid 1-byte range before generation. Acceptance: valid boundary node values are accepted, invalid node values are rejected with documented outcomes, and valid custom epochs can generate IDs.
- **FR-004**: The feature MUST document the uniqueness domain as the combination of custom epoch, node identifier assignment, observed time duration, and sequence value. Acceptance: documentation states when WebeIDs are guaranteed unique and when duplicate risk is the caller's responsibility, including duplicate live generators with the same node.
- **FR-005**: The generator MUST reset the sequence component when the observed millisecond advances and advance the sequence component for additional IDs within the same millisecond. Acceptance: tests show sequence reset across millisecond advancement and monotonic sequence advancement within one millisecond.
- **FR-006**: Default generation MUST fail fast with a typed capacity outcome when the sequence capacity for one node within one millisecond is exhausted. Acceptance: a fixed-time capacity test consumes all sequence values and verifies the next default request returns the documented outcome without a duplicate.
- **FR-007**: The feature MUST detect custom epochs that cannot safely represent current or future generation within the WebeID time range. Acceptance: epoch boundary tests produce documented success or failure outcomes.
- **FR-008**: The feature MUST detect observed time movement that could cause duplicate IDs after generation has started, fail generation while the observed time is behind the last generated duration, and allow generation to resume once observed time reaches or passes that duration. Acceptance: clock-rewind tests verify no ID is emitted while time is behind, the reported outcome identifies the clock safety problem, and generation resumes after catch-up without rebuilding the generator.
- **FR-009**: The feature MUST support restart safety by accepting the full last generated WebeID as the persisted restart marker, deriving the relevant time component from it, and rejecting starts that could reuse prior IDs. Acceptance: restart tests cover a safe last WebeID, a last WebeID equal to the current time component, and a last WebeID from the future relative to the current time component.
- **FR-010**: The feature MUST expose documented, typed outcomes for bad epoch, exhausted time range, bad restart marker, clock rewind, sequence capacity exhaustion, invalid node identifier, and malformed external ID input. Acceptance: each outcome can be matched independently in tests and has a developer-facing explanation.
- **FR-011**: The feature MUST allow a WebeID to be decomposed into its time duration, node identifier, and sequence components without requiring generation state. Acceptance: generated IDs and externally supplied IDs round-trip through decomposition with expected component values.
- **FR-012**: The feature MUST define stable storage and interchange expectations for WebeIDs as numeric and byte-oriented values. Acceptance: documentation and tests show that numeric, big-endian byte, decimal text, and hexadecimal text representations preserve the same components.
- **FR-013**: The feature MUST support documented concurrent usage patterns that allow many request handlers to obtain IDs without duplicates. Acceptance: a concurrent acceptance test generates at least 100,000 successful IDs through the documented pattern and observes zero duplicates.
- **FR-014**: The feature MUST keep generation resource use bounded regardless of how many IDs have previously been generated. Acceptance: a volume test generating at least 1,000,000 IDs shows memory use does not grow with total generated ID count beyond documented fixed state and test harness storage.
- **FR-015**: The feature MUST avoid blocking waits on async execution paths during normal generation. Acceptance: async-workload tests demonstrate successful ID generation from concurrent request-style tasks without blocking the executor under normal load.
- **FR-016**: The feature MUST provide a separate documented bounded backpressure behavior for asynchronous server-style callers that choose to wait for capacity after per-node per-millisecond exhaustion. Acceptance: capacity tests verify server-style callers either receive an ID after safe time advancement within the documented bound or receive the documented timeout/capacity outcome, and no duplicate ID is emitted.
- **FR-017**: The feature MUST include repeatable, self-contained performance measurements for single-generator generation, documented concurrent generation, decomposition, and storage-format conversion without requiring comparison benchmarks against the original WebeID crate or a pass/fail throughput or latency gate. Acceptance: benchmark results are captured before release and include throughput, latency, duplicate-rate observations, and enough environment context to compare future changes.
- **FR-018**: The feature MUST include examples that show generation, decomposition, restart safety, concurrent/server-style usage, and capacity or clock failure handling. Acceptance: every example compiles or is otherwise verified by the documented test process.
- **FR-019**: The feature MUST be reachable through the toolkit facade in a manner consistent with the workspace's existing crate exposure expectations. Acceptance: developers using the toolkit-level interface can reach the documented WebeID capabilities when the feature is enabled.
- **FR-020**: The reimplementation MUST document intentional differences from the original WebeID behavior before release. Acceptance: release-facing documentation lists compatibility notes, changed behavior, and migration guidance for existing users of the old crate.

### Key Entities *(include if feature involves data)*

- **WebeID**: The 64-bit identifier value composed from time duration, node identifier, and sequence components.
- **Custom Epoch**: The developer-selected starting point from which the WebeID time component measures elapsed milliseconds.
- **Time Component**: The 5-byte millisecond duration since the custom epoch, giving the ID its time ordering and representable lifetime.
- **Node Identifier**: The 1-byte value assigned to a generator or deployment participant so independent nodes can generate IDs without coordination for every ID.
- **Sequence Component**: The 2-byte counter that differentiates multiple IDs created by the same node in the same millisecond.
- **Generator**: The stateful facility that observes time, tracks the last generated duration, advances sequences, and emits WebeIDs or documented safety outcomes.
- **Restart Marker**: The persisted full last generated WebeID used to prevent a restarted generator from reusing IDs when the current clock has not advanced far enough beyond that ID's time component.
- **WebeID Components**: The decomposed view of an ID containing its time duration, node identifier, and sequence value.
- **Uniqueness Domain**: The conditions under which generated IDs are guaranteed unique, including epoch selection, node assignment, time movement, sequence capacity, and generator state.
- **Generation Outcome**: A successful WebeID or a typed safety/performance outcome such as bad epoch, clock rewind, restart risk, invalid node, or sequence capacity exhaustion.
- **Performance Profile**: The measured throughput, latency, resource bounds, and concurrency behavior used to validate the feature for server workloads.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In an acceptance run of at least 1,000,000 successful generated IDs using documented generation patterns, 100% of IDs are unique and decompose to expected component values.
- **SC-002**: A reviewer can verify from tests that IDs generated across at least 10,000 increasing time samples sort in the same order as their time components.
- **SC-003**: Boundary tests cover 100% of documented creation and generation outcomes, including bad epoch, exhausted time range, invalid node, bad persisted WebeID restart marker, temporary clock rewind with catch-up recovery, and sequence capacity exhaustion.
- **SC-004**: Documented concurrent usage supports at least 64 simultaneous request-style workers generating at least 100,000 total successful IDs with zero duplicates.
- **SC-005**: Under normal generation load, async request-style workers complete ID generation without blocking waits; default capacity exhaustion fails fast, and the separate server-style backpressure path resolves within the documented bound.
- **SC-006**: Repeatable self-contained performance measurements report single-generator throughput, concurrent-generation throughput, p95 generation latency, decomposition throughput, storage-conversion throughput, and benchmark environment context before implementation is considered complete; no pass/fail performance gate is required for throughput or latency.
- **SC-007**: Volume testing demonstrates that generator-owned memory remains bounded while generating at least 1,000,000 IDs, excluding caller-owned collections used only for validation.
- **SC-008**: A developer new to the feature can generate an ID, decompose it, persist the last generated WebeID for restart safety, and understand the uniqueness limits in under 15 minutes using only the published documentation and examples.
- **SC-009**: Documentation review finds zero undocumented public outcomes within the supported WebeID generation, decomposition, restart, concurrency, and performance scope.

## Assumptions

- The feature will live in this workspace as the WebeID capability commonly referred to as `webe_id`.
- The reimplementation preserves the original WebeID identity model: 5 bytes of custom-epoch milliseconds, 1 byte of node identifier, and 2 bytes of sequence.
- Primary users are hobbyist and small-production developers who want compact, fast, sortable IDs without operating a large distributed ID service.
- Uniqueness across machines or processes depends on correct node identifier assignment by the application or deployment environment.
- Clock rewind and sequence overflow safety mean the feature must never silently emit an ID that could duplicate an earlier WebeID; clock rewind is a temporary failure that can recover after observed time catches up.
- "Tokio friendly" is interpreted as friendly to asynchronous server workloads: normal generation should not create blocking waits for request handlers, and capacity boundaries should have documented bounded behavior.
- Exact implementation choices, dependency choices, and developer-facing interface shape are deferred to planning.
- Performance will be reported with repeatable self-contained measurements for this workspace; comparison benchmarks against the original WebeID crate and pass/fail throughput or latency gates are out of scope.