//! Bounded Tokio backpressure helpers.

use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::{Instant, sleep};

use crate::{GenerateError, Generator, WebeId};

/// Options controlling bounded Tokio-friendly generation backpressure.
///
/// Normal generation never waits. This options type is only for callers that
/// intentionally choose bounded waiting after temporary capacity or clock-rewind
/// failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackpressureOptions {
    timeout: Duration,
    retry_interval: Duration,
}

impl BackpressureOptions {
    /// Creates options with an overall timeout and retry interval.
    pub const fn new(timeout: Duration, retry_interval: Duration) -> Self {
        Self {
            timeout,
            retry_interval,
        }
    }

    /// Returns the maximum time to wait for a recoverable generation condition.
    pub const fn timeout(self) -> Duration {
        self.timeout
    }

    /// Returns the interval between generation attempts.
    pub const fn retry_interval(self) -> Duration {
        self.retry_interval
    }
}

/// Failure returned by bounded Tokio backpressure generation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BackpressureError {
    /// A non-recoverable generation failure occurred.
    Generation(GenerateError),
    /// The timeout expired before recoverable capacity or clock conditions cleared.
    TimedOut {
        /// The configured timeout.
        timeout: Duration,
        /// The last recoverable generation error observed before timeout.
        last_error: GenerateError,
    },
}

impl fmt::Display for BackpressureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generation(error) => write!(formatter, "WebeID generation failed: {error}"),
            Self::TimedOut {
                timeout,
                last_error,
            } => write!(
                formatter,
                "WebeID backpressure timed out after {timeout:?}; last error was {last_error}"
            ),
        }
    }
}

impl Error for BackpressureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Generation(error)
            | Self::TimedOut {
                last_error: error, ..
            } => Some(error),
        }
    }
}

/// Generates a WebeID, waiting only within the configured bound for recoverable states.
///
/// The async mutex is held only while calling [`Generator::generate`]. It is
/// released before sleeping, so no lock is held across an `.await` wait point.
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use std::time::Duration;
/// use tokio::sync::Mutex;
/// use webe_id::async_backpressure::{BackpressureOptions, generate_with_backpressure};
/// use webe_id::{Generator, NodeId};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let generator = Generator::builder(NodeId::from_u8(1)).build()?;
/// let shared = Arc::new(Mutex::new(generator));
/// let id = generate_with_backpressure(
///     shared,
///     BackpressureOptions::new(Duration::from_millis(5), Duration::from_millis(1)),
/// )
/// .await?;
/// assert_eq!(id.components().node_id(), NodeId::from_u8(1));
/// # Ok(())
/// # }
/// ```
pub async fn generate_with_backpressure(
    generator: Arc<Mutex<Generator>>,
    options: BackpressureOptions,
) -> Result<WebeId, BackpressureError> {
    let started = Instant::now();
    let retry_interval = options.retry_interval.max(Duration::from_millis(1));

    loop {
        let result = {
            let mut locked_generator = generator.lock().await;
            locked_generator.generate()
        };

        match result {
            Ok(id) => return Ok(id),
            Err(error) if error.is_temporarily_recoverable() => {
                if started.elapsed() >= options.timeout {
                    return Err(BackpressureError::TimedOut {
                        timeout: options.timeout,
                        last_error: error,
                    });
                }

                let remaining = options.timeout.saturating_sub(started.elapsed());
                sleep(retry_interval.min(remaining)).await;
            }
            Err(error) => return Err(BackpressureError::Generation(error)),
        }
    }
}
