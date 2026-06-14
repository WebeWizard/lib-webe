# Public API Contract: webe_id

The crate's external interface is its public Rust API, surfaced directly as
`webe_id` and through the root facade as `webe::id` when the relevant feature is
enabled. Exact type names, signatures, ownership, and module names are finalized
during implementation; the observable behavior below is binding and maps to the
feature specification.

## Surface Overview

- A WebeID value type representing the canonical 64-bit ID.
- A component/decomposition view exposing time duration, node identifier, and
  sequence.
- A generator configuration path accepting a custom epoch and node identifier.
- A generator creation path that optionally accepts the full last generated
  WebeID for restart safety.
- A default generation operation that emits a WebeID or a typed failure.
- A separate Tokio-friendly bounded backpressure operation for server-style
  callers that choose to wait after sequence capacity exhaustion.
- Stable conversion paths for numeric, big-endian bytes, decimal text, and
  hexadecimal text.
- Typed failures for every documented creation, generation, parsing, restart,
  clock, and capacity outcome.

## Behavioral Contract

### C1 - WebeID layout (FR-001, FR-011, FR-012)

Given a WebeID value, when it is decomposed, then it yields a 40-bit millisecond
duration since the custom epoch, an 8-bit node identifier, and a 16-bit sequence
value in that order. Recomposition from those components returns the same value.

### C2 - Numeric ordering by time (FR-002)

Given IDs generated with increasing time components in the same uniqueness
domain, when sorted numerically, then the IDs sort in the same order as their time
components.

### C3 - Generator creation (FR-003, FR-007)

Given a valid custom epoch and node identifier, when a generator is created, then
it is ready to generate IDs. Given an invalid node, an epoch in the future, or a
current duration outside the WebeID time range, creation returns a typed failure.

### C4 - Same-millisecond sequencing (FR-005)

Given a generator observes the same millisecond for multiple default generation
requests, when sequence capacity remains, then each successful ID has the same
time and node components and a distinct advancing sequence component.

### C5 - Sequence reset after time advancement (FR-005)

Given a generator observes a later millisecond than the last generated ID, when it
generates the next ID, then the sequence component resets to the first sequence
value for that millisecond.

### C6 - Default capacity exhaustion (FR-006)

Given one node has emitted all 65,536 sequence values for an observed millisecond,
when default generation is requested again before time advances, then generation
fails fast with a typed capacity outcome and emits no ID.

### C7 - Bounded server-style backpressure (FR-016)

Given one node has exhausted sequence capacity in the current millisecond, when a
caller uses the documented Tokio-friendly bounded backpressure operation, then it
either emits an ID after safe time advancement within the documented bound or
returns the documented timeout/capacity outcome. It must not spin without bound,
grow memory without bound, or emit a duplicate.

### C8 - Temporary clock rewind recovery (FR-008)

Given observed time moves behind the last generated duration, when generation is
requested, then no ID is emitted and a typed clock-rewind outcome is returned.
Given observed time later reaches or passes the last generated duration, when
generation is requested again, then generation can resume without rebuilding the
generator.

### C9 - Restart marker safety (FR-009)

Given the full last generated WebeID from a previous run, when a new generator is
created with the same custom epoch and node, then creation succeeds only if the
current observed duration is greater than that marker's time component. If the
marker node is incompatible, the marker is malformed, or the current duration has
not advanced far enough, creation returns a typed restart-safety failure.

### C10 - External representation parsing (FR-010, FR-012)

Given numeric, big-endian byte, decimal text, or hexadecimal text input, when it
represents a valid WebeID, then parsing succeeds and decomposition yields the same
components. Malformed input returns a typed malformed-input failure.

### C11 - Concurrent documented usage (FR-013)

Given many request-style workers use the documented shared generation pattern,
when at least 100,000 successful IDs are generated, then all successful IDs are
unique. Capacity and clock failures remain typed outcomes rather than panics.

### C12 - Bounded memory (FR-014)

Given a generator emits a large number of IDs over time, when caller-owned
validation collections are excluded, then generator-owned memory remains bounded
by fixed state rather than growing with total generated ID count.

### C13 - Facade reachability (FR-019)

Given the root crate is built with the WebeID feature enabled, when a developer
uses `webe::id`, then the documented WebeID value, generator, component, error,
and optional Tokio-friendly capabilities are reachable through the facade feature
structure.

### C14 - Compatibility documentation (FR-020)

Given any behavior intentionally differs from the original WebeID repository,
when release-facing documentation is reviewed, then the difference and migration
impact are documented before release.

## Non-Functional Contract

- **NF1 (Quality)**: Public items are documented; library code avoids `unsafe`,
  `.unwrap()`, and `.expect()`; errors are typed and actionable.
- **NF2 (Hot-path performance)**: Default generation, decomposition, and byte
  conversion are O(1), keep generator-owned memory bounded, and avoid per-ID heap
  allocation.
- **NF3 (Async friendliness)**: Normal generation performs no blocking waits.
  Waiting behavior exists only in the separate bounded server-style path.
- **NF4 (Performance reporting)**: Benchmarks report throughput, latency,
  duplicate-rate observations, and environment context. No comparison benchmark
  against the original WebeID crate and no pass/fail throughput or latency gate
  are required.

## Verification Mapping

Every contract item above must be covered by automated tests or benchmark-report
verification as described in [quickstart.md](../quickstart.md). The root facade
path is covered separately from direct crate usage.