# webe_id

`webe_id` provides compact, sortable 64-bit identifiers for the Webe toolkit.
The root `webe` crate re-exports this crate as `webe::id` behind the `id`
feature, with optional Tokio backpressure helpers behind `id-tokio`.

## Layout

A WebeID is one canonical `u64` value with byte-aligned components:

```text
| 40-bit milliseconds since epoch | 8-bit node | 16-bit sequence |
```

Numeric ordering follows the time component because milliseconds occupy the high
bits. The same value round-trips through `u64`, big-endian bytes, decimal text,
and fixed-width lowercase hexadecimal text.

## Basic Usage

```rust
use webe_id::{Generator, NodeId};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut generator = Generator::builder(NodeId::from_u8(7)).build()?;
let id = generator.generate()?;
let parts = id.components();

assert_eq!(parts.node_id(), NodeId::from_u8(7));
println!("{} {}", id.to_decimal_string(), id.to_hex_string());
# Ok(())
# }
```

For a domain-specific epoch, configure one explicitly:

```rust
use webe_id::time::default_epoch;
use webe_id::{Generator, NodeId};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut generator = Generator::builder(NodeId::from_u8(1))
	.with_epoch(default_epoch())
		.build()?;
let _id = generator.generate()?;
# Ok(())
# }
```

The builder default epoch is `2025-01-01T00:00:00Z`. Persist the epoch choice
with your application configuration; restart markers only make sense in the same
epoch and uniqueness domain.

## Uniqueness Domain

WebeID uniqueness is guaranteed by the tuple of custom epoch, node ID, observed
millisecond, and sequence value. Callers must assign distinct `NodeId` values to
concurrently active nodes in the same domain. Values `0` and `255` are valid;
external configuration can be checked with `NodeId::new(value)`.

For multi-threaded synchronous use, share a generator behind a lock:

```rust
use std::sync::{Arc, Mutex};
use webe_id::{Generator, NodeId};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let generator = Generator::builder(NodeId::from_u8(2)).build()?;
let shared = Arc::new(Mutex::new(generator));
let id = shared.lock().map_err(|_| "lock poisoned")?.generate()?;
assert_eq!(id.components().node_id(), NodeId::from_u8(2));
# Ok(())
# }
```

## Restart Markers

Persist the full last generated WebeID if a process may restart quickly on the
same node. On startup, pass that value to the builder:

```rust
use webe_id::{Generator, NodeId, WebeIdComponents};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let last_id = WebeIdComponents::new(1, NodeId::from_u8(1), 0)?.to_id();
let generator = Generator::builder(NodeId::from_u8(1))
		.with_restart_marker(last_id)
		.build()?;
assert_eq!(generator.node_id(), NodeId::from_u8(1));
# Ok(())
# }
```

The marker node must match the configured node, and the current observed duration
must be strictly greater than the marker's time component. Equal-time restarts
are rejected because they could reuse sequence values.

## Capacity And Clock Limits

- Time range: `0..=2^40 - 1` milliseconds after the configured epoch.
- Node range: `0..=255`.
- Sequence range: `0..=65,535` per node per millisecond.
- Default generation fails fast after 65,536 IDs in one millisecond.
- A temporary clock rewind returns a typed error and recovers when observed time
	reaches or passes the last generated millisecond.

Normal generation does not wait or block for time to advance. It returns typed
errors so request paths can decide whether to retry, surface pressure, or use the
optional bounded async path.

## Tokio Backpressure

Enable the `tokio` feature on `webe_id` or `id-tokio` on the root `webe` crate to
use bounded async backpressure:

```rust,no_run
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use webe_id::async_backpressure::{BackpressureOptions, generate_with_backpressure};
use webe_id::{Generator, NodeId};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let generator = Generator::builder(NodeId::from_u8(1)).build()?;
let shared = Arc::new(Mutex::new(generator));
let id = generate_with_backpressure(
		shared,
		BackpressureOptions::new(Duration::from_millis(5), Duration::from_millis(1)),
)
.await?;
assert_eq!(id.components().node_id(), NodeId::from_u8(1));
# Ok(())
# }
```

The helper locks only long enough to call `Generator::generate()`, releases the
lock before sleeping, and returns `BackpressureError::TimedOut` if the configured
bound expires before safe generation resumes.

## Errors

The public error types are directly matchable:

- `NodeIdError` for invalid node configuration.
- `WebeIdComponentsError` for out-of-range component recomposition.
- `ParseWebeIdError` for malformed byte, decimal, or hexadecimal input.
- `BuildGeneratorError` for epoch, range, node, and restart-marker failures.
- `GenerateError` for clock rewind, time range, and sequence capacity failures.
- `async_backpressure::BackpressureError` when the Tokio feature is enabled.

Error messages are intended for developer logs; use enum variants for program
control flow.

## Benchmark Reporting

Run the reporting-only benchmark harness with:

```bash
cargo bench -p webe_id
```

The report includes package version, OS, architecture, logical CPU count, Rust
toolchain, active Tokio feature state, single-generator throughput, p95
generation latency, concurrent throughput, duplicate-rate observation,
decomposition throughput, and conversion throughput. It does not compare against
the original WebeID repository and does not enforce pass/fail performance gates.

## Compatibility Notes

This implementation preserves the compact WebeID concept and the 5-byte time,
1-byte node, 2-byte sequence layout. Intentional operational choices are:

- Core generation is standard-library-only; Tokio support is optional.
- Default generation fails fast on sequence exhaustion instead of waiting.
- Bounded waiting is explicit through the Tokio helper.
- Restart safety uses the full last generated WebeID as the marker.
- The builder has a documented default epoch, but production systems should
	choose and persist their own epoch when long-lived domains matter.