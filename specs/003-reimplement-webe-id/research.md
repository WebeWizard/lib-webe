# Phase 0 Research: Reimplement Webe ID

All five clarifications were resolved in the spec. There are no remaining open
clarification placeholders. This document records the technical decisions that
shape Phase 1 design.

## Decision: Add a new `webe_id` crate and facade feature

- **Decision**: Implement WebeID as a new workspace member at `crates/webe_id`,
  surface it from the root crate as `webe::id`, and add root feature wiring for
  the core ID feature plus a Tokio-enabled variant for bounded async
  backpressure.
- **Rationale**: The README already identifies unique ID generation as a toolkit
  capability currently living outside the workspace. A dedicated crate keeps the
  ID domain self-contained and matches existing workspace organization
  (`webe_args`, `webe_log`, `webe_web`). The root facade remains consistent with
  constitution UX requirements.
- **Alternatives considered**: Place the generator in `webe_web` because request
  paths may need IDs - rejected because WebeID is not HTTP-specific. Place the
  implementation directly in the root `webe` crate - rejected because the facade
  should re-export focused implementation crates.

## Decision: Preserve the byte-aligned 64-bit WebeID layout

- **Decision**: The canonical layout is 40 bits of milliseconds since a custom
  epoch in the high-order bytes, followed by 8 bits of node identifier, followed
  by 16 bits of sequence. Numeric values, big-endian bytes, decimal text, and
  hexadecimal text must all round-trip through this same component model.
- **Rationale**: This is the defining concept from the original WebeID: compact,
  sortable IDs with byte-boundary components that are easy to decompose from
  big-endian bytes. Preserving the layout satisfies FR-001, FR-002, FR-011, and
  FR-012.
- **Alternatives considered**: A Snowflake-style 41/10/12 bit split - rejected
  because it changes the WebeID concept and node/sequence capacity. A 128-bit ID
  - rejected because compact 64-bit identity is central to the feature.

## Decision: Keep core generation standard-library-only

- **Decision**: The core generator uses only the Rust standard library. Optional
  Tokio support is isolated to a feature-gated bounded backpressure path, and
  performance reporting uses dev-only benchmark tooling.
- **Rationale**: Default generation should stay tiny, fast, and usable outside an
  async runtime. The workspace already carries Tokio for web work, so an optional
  Tokio path can support server workloads without putting runtime behavior on the
  hot path for every caller.
- **Alternatives considered**: Make Tokio a required dependency - rejected as
  unnecessary for fail-fast generation and decomposition. Avoid Tokio entirely -
  rejected because the spec explicitly asks for async/server-workload friendly
  bounded backpressure.

## Decision: Use deterministic time control for tests and boundary behavior

- **Decision**: The generator design includes an internal time-source boundary so
  tests can drive exact millisecond durations, clock rewinds, catch-up recovery,
  exhausted time range, and sequence exhaustion without waiting on real wall
  time. The public API exposes normal system-clock construction unless a test or
  advanced path explicitly needs deterministic control.
- **Rationale**: FR-005 through FR-009 and SC-003 require precise boundary tests.
  Real-time sleeps would make the suite slow and flaky, especially around
  sequence capacity and clock rewind recovery.
- **Alternatives considered**: Test only with `SystemTime::now()` - rejected as
  nondeterministic. Expose broad clock injection as the primary API - rejected as
  extra public surface for ordinary users.

## Decision: Fail fast by default, provide separate bounded async backpressure

- **Decision**: Default generation returns a typed capacity outcome immediately
  after 65,536 IDs are requested by one node in one millisecond. A separate
  documented Tokio-friendly path may wait for safe time advancement, but only
  within a caller-visible bound.
- **Rationale**: The hot path remains fast and non-blocking, satisfying the
  user's performance emphasis and FR-006. Server-style callers still get a safe
  alternative when they prefer bounded waiting over handling capacity errors.
- **Alternatives considered**: Always wait for the next millisecond - rejected
  because it would hide blocking behavior in default generation. Only fail fast
  with no async path - rejected because it would not satisfy FR-016.

## Decision: Treat clock rewind as temporary until catch-up

- **Decision**: If observed time falls behind the last generated duration, the
  generator refuses to emit IDs with a typed clock-rewind outcome. Once observed
  time reaches or passes the last generated duration, generation may resume
  without rebuilding the generator.
- **Rationale**: This prevents duplicate IDs while avoiding a permanent outage
  for transient NTP, VM, container, or sleep/wake clock adjustments.
- **Alternatives considered**: Permanent failed state after any rewind - rejected
  as too operationally harsh for temporary drift. Synthetic monotonic time -
  rejected because it weakens the meaning of the WebeID time component.

## Decision: Persist the full last WebeID as the restart marker

- **Decision**: Restart safety accepts the full last generated WebeID, derives its
  time and node components, verifies compatibility with the new generator, and
  rejects startup when the current duration has not advanced beyond the marker's
  time component.
- **Rationale**: Persisting a full ID is easy for callers to store and audit, and
  it carries the component data needed to protect against reuse after restart.
  This directly implements the final clarification and FR-009.
- **Alternatives considered**: Persist only the last duration - rejected because
  it is less ergonomic and less auditable. Support many marker forms in v1 -
  rejected to keep the contract small and testable.

## Decision: Performance is reported, not gated

- **Decision**: The crate includes repeatable self-contained measurements for
  default generation, documented concurrent generation, decomposition, and
  storage conversion. Results include throughput, p95 latency where practical,
  duplicate-rate observations, and environment context. The plan does not require
  comparison benchmarks against the original WebeID crate and does not set a
  pass/fail throughput or latency gate.
- **Rationale**: The user explicitly rejected comparison benchmarks and later
  selected reporting-only performance. Reporting still satisfies the
  constitution's requirement that performance-sensitive work be backed by data,
  and gives maintainers a baseline for future regression review.
- **Alternatives considered**: Match or beat the original crate - rejected by user
  clarification. A fixed IDs/sec threshold - rejected by user clarification. No
  benchmarks at all - rejected because performance is a first-class concern.