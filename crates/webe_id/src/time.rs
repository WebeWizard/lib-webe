//! Time range and clock-source utilities.

use std::time::{Duration, SystemTime};

use crate::id::MAX_TIME_MILLISECONDS;

/// Maximum duration since a custom epoch that can fit in the 40-bit time field.
pub const MAX_DURATION_MILLISECONDS: u64 = MAX_TIME_MILLISECONDS;

/// Unix timestamp, in seconds, of the default WebeID epoch: 2025-01-01T00:00:00Z.
pub const DEFAULT_EPOCH_UNIX_SECONDS: u64 = 1_735_689_600;

/// Returns the default WebeID epoch used by [`crate::GeneratorBuilder`].
pub fn default_epoch() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(DEFAULT_EPOCH_UNIX_SECONDS)
}

/// Source of wall-clock time for a WebeID generator.
///
/// Implement this trait in tests to drive exact millisecond values. Production
/// code normally uses [`SystemClock`] through [`crate::GeneratorBuilder`]'s
/// default configuration.
pub trait Clock: Send + Sync + 'static {
    /// Returns the currently observed wall-clock time.
    fn now(&self) -> SystemTime;
}

/// System clock implementation used by default generator builders.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

/// Failure returned when an observed time cannot be represented as a WebeID duration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TimeRangeError {
    /// The observed time is earlier than the configured epoch.
    BeforeEpoch,
    /// The elapsed duration exceeds the 40-bit millisecond field.
    ExceedsMaximum {
        /// The rejected elapsed millisecond duration.
        elapsed_millis: u128,
        /// The maximum elapsed millisecond duration that can be encoded.
        max_millis: u64,
    },
}

/// Computes checked elapsed milliseconds between a custom epoch and observed time.
pub fn checked_milliseconds_since_epoch(
    epoch: SystemTime,
    observed: SystemTime,
) -> Result<u64, TimeRangeError> {
    let duration = observed
        .duration_since(epoch)
        .map_err(|_| TimeRangeError::BeforeEpoch)?;
    let elapsed_millis = duration.as_millis();

    if elapsed_millis > u128::from(MAX_DURATION_MILLISECONDS) {
        return Err(TimeRangeError::ExceedsMaximum {
            elapsed_millis,
            max_millis: MAX_DURATION_MILLISECONDS,
        });
    }

    Ok(elapsed_millis as u64)
}
