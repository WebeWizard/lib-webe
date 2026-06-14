# Data Model: Reimplement Webe ID

## WebeID

Represents the canonical 64-bit identifier value.

- **Fields**:
  - `raw`: 64-bit unsigned numeric value.
  - `time_component_ms`: 40-bit millisecond duration since the custom epoch.
  - `node_id`: 8-bit node identifier.
  - `sequence`: 16-bit per-node, per-millisecond sequence value.
- **Validation rules**:
  - Time component is in `0..=2^40 - 1` milliseconds.
  - Node component is in `0..=255`.
  - Sequence component is in `0..=65,535`.
  - Big-endian bytes, decimal text, and hexadecimal text must round-trip to the
    same raw value and components.
- **Relationships**:
  - Created from `Generator` state or parsed from external representations.
  - Decomposes into `WebeID Components` without generation state.

## WebeID Components

Represents the decomposed view of a WebeID.

- **Fields**:
  - `time_component_ms`: elapsed milliseconds since the custom epoch.
  - `node_id`: node that generated or is encoded by the ID.
  - `sequence`: sequence value within the same node and millisecond.
- **Validation rules**:
  - Component values must fit the WebeID layout boundaries.
  - Recomposition from components must produce the original WebeID.
- **Relationships**:
  - Used by restart marker validation, documentation examples, and tests.

## Custom Epoch

Represents the developer-selected time origin for a WebeID domain.

- **Fields**:
  - `instant`: wall-clock timestamp used as duration origin.
  - `max_duration_ms`: fixed maximum duration representable by the 40-bit time
    component.
- **Validation rules**:
  - The current observed time must not be before the epoch for normal generation.
  - The current observed duration must not exceed the maximum representable
    WebeID time range.
  - Documentation must state that restart markers are interpreted relative to the
    same custom epoch.
- **Relationships**:
  - Owned by a `Generator`.
  - Defines the time meaning of all WebeIDs generated in the same uniqueness
    domain.

## Node Identifier

Represents the node or deployment participant assigned to a generator.

- **Fields**:
  - `value`: 8-bit node identifier.
- **Validation rules**:
  - Values `0` and `255` are valid boundaries.
  - Values outside `0..=255` from external configuration are rejected with a typed
    invalid-node outcome.
- **Relationships**:
  - Encoded into every WebeID from a generator.
  - Must be assigned uniquely by the caller across concurrently active nodes in
    the same WebeID uniqueness domain.
  - Must match the persisted restart marker's node component when restart safety
    is requested.

## Sequence Component

Represents the per-node counter within one millisecond.

- **Fields**:
  - `value`: 16-bit sequence number.
- **Validation rules**:
  - Starts at `0` for the first ID in a newly observed millisecond.
  - Increments for each additional ID in the same observed millisecond.
  - Exhaustion after `65,535` causes default generation to fail fast rather than
    wrap.
- **Relationships**:
  - Owned by `Generator` state.
  - Combined with time and node components to form a WebeID.

## Generator

Represents the stateful facility that emits WebeIDs.

- **Fields**:
  - `epoch`: custom epoch.
  - `node_id`: configured node identifier.
  - `last_duration_ms`: last duration used to emit a WebeID, if generation has
    occurred.
  - `next_sequence`: next sequence value for the current millisecond.
  - `clock_source`: source of observed time, deterministic in tests and system
    time in normal use.
- **Validation rules**:
  - Creation validates epoch and node boundaries.
  - Creation with a restart marker validates that the marker node matches the
    configured node and the marker time component is strictly behind current
    observed duration.
  - Generation refuses to emit while observed duration is behind
    `last_duration_ms`.
  - Generation resumes after a temporary clock rewind once observed duration
    reaches or passes `last_duration_ms`.
  - Default generation fails fast when the current millisecond has no remaining
    sequence values.
  - Bounded async backpressure either emits after safe time advancement within the
    documented bound or reports the documented timeout/capacity outcome.
- **Relationships**:
  - Produces `WebeID` values or `Generation Outcome` failures.
  - Used through documented shared/concurrent usage patterns for server workers.

## Restart Marker

Represents the persisted full last generated WebeID used to prevent reuse after a
process restart.

- **Fields**:
  - `last_webe_id`: full WebeID value from the previous run.
  - Derived `time_component_ms` and `node_id`.
- **Validation rules**:
  - Marker must parse as a valid WebeID.
  - Marker node component must match the new generator's configured node.
  - Current observed duration must be greater than the marker time component.
  - Marker must be documented as valid only for the same custom epoch and
    uniqueness domain.
- **Relationships**:
  - Optional input to `Generator` creation.
  - Produces a typed restart-safety failure when invalid or unsafe.

## Generation Outcome

Represents either a successful WebeID or a typed safety/input failure.

- **Fields**:
  - `success`: generated WebeID.
  - `failure`: one of bad epoch, exhausted time range, invalid node, bad restart
    marker, clock rewind, sequence capacity exhaustion, malformed external ID
    input, or bounded backpressure timeout/capacity.
- **Validation rules**:
  - Failures are matchable by category.
  - Developer-facing messages identify what failed and why it matters for
    uniqueness or representation.
- **Relationships**:
  - Returned by generator creation, generation, parsing, and bounded backpressure
    operations.

## Performance Profile

Represents reported measurement data for maintainers.

- **Fields**:
  - Single-generator throughput and latency observations.
  - Documented concurrent-generation throughput and latency observations.
  - Decomposition throughput observations.
  - Storage-format conversion throughput observations.
  - Duplicate-rate observation for measured successful IDs.
  - Hardware, OS, toolchain, feature flags, and command context.
- **Validation rules**:
  - Reporting is self-contained and repeatable.
  - No comparison to the original WebeID crate is required.
  - No pass/fail throughput or latency gate is required for acceptance.
- **Relationships**:
  - Referenced by release notes or change descriptions for performance-sensitive
    updates.

## State Transitions

```text
Not Created
  -> Ready
     when epoch, node, and optional restart marker validate

Ready
  -> Ready
     when observed duration advances; sequence resets to 0 and ID is emitted

Ready
  -> Same Millisecond
     when observed duration equals last duration and sequence remains available

Same Millisecond
  -> Same Millisecond
     when another ID is emitted and sequence remains available

Same Millisecond
  -> Capacity Exhausted
     when all 65,536 sequence values are consumed in one millisecond

Capacity Exhausted
  -> Ready
     when observed duration advances and default generation is retried

Capacity Exhausted
  -> Backpressure Waiting
     when server-style bounded backpressure is requested

Backpressure Waiting
  -> Ready
     when safe time advancement occurs within the documented bound

Backpressure Waiting
  -> Capacity/Timeout Outcome
     when the documented bound expires before safe time advancement

Ready or Same Millisecond
  -> Temporary Clock Rewind
     when observed duration is behind last generated duration

Temporary Clock Rewind
  -> Ready
     when observed duration reaches or passes last generated duration
```