//! Generator state machine and construction helpers.

use std::fmt;
use std::sync::Arc;
use std::time::SystemTime;

use crate::components::WebeIdComponents;
use crate::error::{BuildGeneratorError, GenerateError};
use crate::id::{MAX_SEQUENCE, MAX_TIME_MILLISECONDS, WebeId};
use crate::node::NodeId;
use crate::time::{
    Clock, SystemClock, TimeRangeError, checked_milliseconds_since_epoch, default_epoch,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MillisecondState {
    time_millis: u64,
    next_sequence: u32,
}

/// Stateful bounded-memory WebeID generator.
///
/// The generator stores fixed state only: a custom epoch, node ID, clock source,
/// and the last observed millisecond with its next sequence value. It performs no
/// per-ID heap allocation during default generation.
///
/// For concurrent use, share a generator behind a synchronization primitive such
/// as `std::sync::Mutex` for synchronous code. Tokio callers that choose bounded
/// waiting can use the `tokio` feature's backpressure helper, which releases the
/// async lock before every wait.
///
/// # Examples
///
/// ```
/// use webe_id::{Generator, NodeId};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut generator = Generator::builder(NodeId::from_u8(1)).build()?;
/// let id = generator.generate()?;
/// assert_eq!(id.components().node_id(), NodeId::from_u8(1));
/// # Ok(())
/// # }
/// ```
///
/// Persist and provide the full last WebeID when restart safety matters:
///
/// ```
/// use webe_id::time::default_epoch;
/// use webe_id::{Generator, NodeId, WebeIdComponents};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let marker = WebeIdComponents::new(1, NodeId::from_u8(1), 0)?.to_id();
/// let generator = Generator::builder(NodeId::from_u8(1))
///     .with_epoch(default_epoch())
///     .with_restart_marker(marker)
///     .build()?;
/// assert_eq!(generator.node_id(), NodeId::from_u8(1));
/// # Ok(())
/// # }
/// ```
pub struct Generator {
    epoch: SystemTime,
    node_id: NodeId,
    clock: Arc<dyn Clock>,
    last: Option<MillisecondState>,
}

impl Generator {
    /// Starts a builder for a generator with a validated node ID.
    pub fn builder(node_id: NodeId) -> GeneratorBuilder {
        GeneratorBuilder::new(node_id)
    }

    /// Starts a builder from an external node configuration value.
    pub fn builder_from_node_value(value: u16) -> Result<GeneratorBuilder, BuildGeneratorError> {
        Ok(Self::builder(NodeId::new(value)?))
    }

    /// Creates a generator with the default Unix epoch and system clock.
    pub fn new(node_id: NodeId) -> Result<Self, BuildGeneratorError> {
        Self::builder(node_id).build()
    }

    /// Returns the configured node identifier.
    pub const fn node_id(&self) -> NodeId {
        self.node_id
    }

    /// Returns the configured custom epoch.
    pub const fn epoch(&self) -> SystemTime {
        self.epoch
    }

    /// Generates the next WebeID or returns a typed safety failure.
    pub fn generate(&mut self) -> Result<WebeId, GenerateError> {
        let observed_millis = self.observed_millis()?;

        let sequence = match self.last {
            Some(last) if observed_millis < last.time_millis => {
                return Err(GenerateError::ClockRewind {
                    observed_millis,
                    last_millis: last.time_millis,
                });
            }
            Some(last) if observed_millis == last.time_millis => {
                if last.next_sequence > u32::from(MAX_SEQUENCE) {
                    return Err(GenerateError::SequenceCapacityExhausted {
                        time_millis: observed_millis,
                    });
                }

                let sequence = last.next_sequence as u16;
                self.last = Some(MillisecondState {
                    time_millis: observed_millis,
                    next_sequence: last.next_sequence + 1,
                });
                sequence
            }
            Some(_) | None => {
                self.last = Some(MillisecondState {
                    time_millis: observed_millis,
                    next_sequence: 1,
                });
                0
            }
        };

        Ok(
            WebeIdComponents::new(observed_millis, self.node_id, sequence)
                .map_err(|_| GenerateError::TimeRangeExceeded {
                    elapsed_millis: u128::from(observed_millis),
                    max_millis: MAX_TIME_MILLISECONDS,
                })?
                .to_id(),
        )
    }

    fn observed_millis(&self) -> Result<u64, GenerateError> {
        checked_milliseconds_since_epoch(self.epoch, self.clock.now()).map_err(
            |error| match error {
                TimeRangeError::BeforeEpoch => GenerateError::EpochInFuture,
                TimeRangeError::ExceedsMaximum {
                    elapsed_millis,
                    max_millis,
                } => GenerateError::TimeRangeExceeded {
                    elapsed_millis,
                    max_millis,
                },
            },
        )
    }
}

impl fmt::Debug for Generator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Generator")
            .field("epoch", &self.epoch)
            .field("node_id", &self.node_id)
            .field("last", &self.last)
            .finish_non_exhaustive()
    }
}

/// Builder for configuring a WebeID generator.
pub struct GeneratorBuilder {
    epoch: SystemTime,
    node_id: NodeId,
    clock: Arc<dyn Clock>,
    restart_marker: Option<WebeId>,
}

impl GeneratorBuilder {
    fn new(node_id: NodeId) -> Self {
        Self {
            epoch: default_epoch(),
            node_id,
            clock: Arc::new(SystemClock),
            restart_marker: None,
        }
    }

    /// Sets the custom epoch used as the origin for generated time components.
    pub fn with_epoch(mut self, epoch: SystemTime) -> Self {
        self.epoch = epoch;
        self
    }

    /// Sets the clock source used by the generator.
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Sets the full last generated WebeID from a previous run for restart safety.
    pub fn with_restart_marker(mut self, marker: WebeId) -> Self {
        self.restart_marker = Some(marker);
        self
    }

    /// Validates the configuration and constructs a ready generator.
    pub fn build(self) -> Result<Generator, BuildGeneratorError> {
        let current_millis = checked_milliseconds_since_epoch(self.epoch, self.clock.now())
            .map_err(|error| match error {
                TimeRangeError::BeforeEpoch => BuildGeneratorError::EpochInFuture,
                TimeRangeError::ExceedsMaximum {
                    elapsed_millis,
                    max_millis,
                } => BuildGeneratorError::TimeRangeExceeded {
                    elapsed_millis,
                    max_millis,
                },
            })?;

        if let Some(marker) = self.restart_marker {
            let components = marker.components();
            if components.node_id() != self.node_id {
                return Err(BuildGeneratorError::RestartMarkerNodeMismatch {
                    marker_node: components.node_id(),
                    generator_node: self.node_id,
                });
            }

            if current_millis <= components.time_millis() {
                return Err(BuildGeneratorError::RestartMarkerNotBehindCurrentTime {
                    marker_millis: components.time_millis(),
                    current_millis,
                });
            }
        }

        Ok(Generator {
            epoch: self.epoch,
            node_id: self.node_id,
            clock: self.clock,
            last: None,
        })
    }
}

impl fmt::Debug for GeneratorBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GeneratorBuilder")
            .field("epoch", &self.epoch)
            .field("node_id", &self.node_id)
            .field("restart_marker", &self.restart_marker)
            .finish_non_exhaustive()
    }
}
