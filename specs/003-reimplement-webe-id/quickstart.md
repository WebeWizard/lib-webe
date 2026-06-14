# Quickstart: Reimplement Webe ID

This guide defines the validation scenarios for the WebeID implementation plan.
It is a run guide, not implementation code.

## Prerequisites

- Rust stable from `rust-toolchain.toml`.
- Workspace dependencies resolved by Cargo.
- The builder default epoch is 2025-01-01T00:00:00Z; applications that need a
   domain-specific epoch should configure and persist that epoch explicitly.
- No MySQL client libraries are required for WebeID validation. Exclude
  `webe_auth` from whole-workspace checks in environments without MySQL client
  libraries.

## Setup Checks

```bash
cargo fmt --check
cargo clippy -p webe_id --all-targets -- -D warnings
cargo clippy -p webe_id --features tokio --all-targets -- -D warnings
cargo test -p webe_id
cargo test -p webe_id --features tokio
cargo test -p webe --features id,id-tokio
cargo test --workspace --exclude webe_auth
```

Expected outcome: formatting is clean, clippy reports no warnings for direct and
Tokio-feature builds, direct crate tests pass, Tokio-feature tests pass, facade
tests pass, and the workspace test run passes excluding `webe_auth` where system
MySQL client libraries are absent.

## Scenario 1: Layout and Decomposition

1. Create a generator with a known custom epoch and node identifier.
2. Generate WebeIDs across controlled milliseconds.
3. Decompose each ID into time, node, and sequence components.
4. Convert each ID through numeric, big-endian bytes, decimal text, and
   hexadecimal text.

Expected outcome: components match the 5-byte time, 1-byte node, and 2-byte
sequence layout; increasing time components sort numerically in the same order;
all supported representations round-trip to the same ID.

## Scenario 2: Same-Millisecond Capacity

1. Hold the generator's observed time at one millisecond.
2. Generate all 65,536 sequence values for one node.
3. Request one additional default ID before observed time advances.
4. Advance observed time by one millisecond and request another default ID.

Expected outcome: all 65,536 successful IDs are unique; the additional default
request fails fast with the typed capacity outcome; after time advances, the next
successful ID uses the new time component and reset sequence.

## Scenario 3: Bounded Tokio-Friendly Backpressure

1. Enable the Tokio feature.
2. Exhaust the current millisecond's sequence capacity.
3. Request an ID through the documented bounded server-style backpressure path.
4. Run both a case where observed time advances inside the bound and a case where
   it does not.

Expected outcome: the first case returns a unique ID after safe time advancement;
the second case returns the documented timeout/capacity outcome within the bound;
neither case emits a duplicate or blocks the normal generation path.

## Scenario 4: Clock Rewind and Recovery

1. Generate an ID at a controlled duration.
2. Move observed time behind that last generated duration.
3. Request another ID.
4. Move observed time to the last generated duration or later.
5. Request another ID.

Expected outcome: while time is behind, generation returns a typed clock-rewind
outcome and emits no ID. After catch-up, generation resumes without rebuilding
the generator and without reusing an earlier WebeID.

## Scenario 5: Restart Safety

1. Generate an ID and persist that full WebeID as the last generated ID.
2. Create a new generator with the same custom epoch and node after observed time
   has advanced beyond the persisted ID's time component.
3. Repeat with observed time equal to the persisted ID's time component.
4. Repeat with a persisted ID whose node component does not match the new
   generator's node.

Expected outcome: the advanced-time restart succeeds; equal-time and
future-time restarts fail with a typed restart-safety outcome; incompatible node
markers fail with a typed bad-marker outcome.

## Scenario 6: Concurrent Server-Style Usage

1. Use the documented shared generation pattern from many request-style workers.
2. Generate at least 100,000 successful IDs.
3. Collect successful IDs only in the test harness for duplicate checking.

Expected outcome: all successful IDs are unique; generator-owned memory stays
bounded; any capacity or clock safety failures are typed outcomes rather than
panics.

## Scenario 7: Performance Reporting

```bash
cargo bench -p webe_id
```

Expected outcome: benchmark output reports single-generator throughput,
concurrent-generation throughput, p95 generation latency where supported by the
harness, decomposition throughput, storage-conversion throughput, duplicate-rate
observations, and environment context including package version, OS,
architecture, logical CPU count, active feature state, and Rust toolchain. The
report does not compare against the original WebeID crate and does not enforce a
pass/fail throughput or latency gate.

## Release Documentation Check

Before release, review:

- `crates/webe_id/README.md`
- root `README.md`
- rustdoc examples for public WebeID types and errors
- compatibility notes for intentional differences from the original repository

Expected outcome: a new developer can generate an ID, decompose it, persist the
last generated WebeID for restart safety, understand uniqueness limits, and find
the Tokio-friendly bounded path in under 15 minutes.