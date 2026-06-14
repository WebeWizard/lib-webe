//! Component-level WebeID helpers.

use crate::error::WebeIdComponentsError;
use crate::id::{MAX_TIME_MILLISECONDS, NODE_SHIFT, TIME_SHIFT, WebeId};
use crate::node::NodeId;

/// Decomposed WebeID components.
///
/// # Examples
///
/// ```
/// use webe_id::{NodeId, WebeIdComponents};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let components = WebeIdComponents::new(42, NodeId::from_u8(7), 3)?;
/// let id = components.to_id();
/// assert_eq!(id.components(), components);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WebeIdComponents {
    time_millis: u64,
    node_id: NodeId,
    sequence: u16,
}

impl WebeIdComponents {
    /// Creates validated components that can be encoded into a WebeID.
    pub fn new(
        time_millis: u64,
        node_id: NodeId,
        sequence: u16,
    ) -> Result<Self, WebeIdComponentsError> {
        if time_millis > MAX_TIME_MILLISECONDS {
            return Err(WebeIdComponentsError::TimeRangeExceeded {
                time_millis,
                max_millis: MAX_TIME_MILLISECONDS,
            });
        }

        Ok(Self {
            time_millis,
            node_id,
            sequence,
        })
    }

    /// Returns the elapsed milliseconds since the custom epoch.
    pub const fn time_millis(self) -> u64 {
        self.time_millis
    }

    /// Returns the node identifier component.
    pub const fn node_id(self) -> NodeId {
        self.node_id
    }

    /// Returns the per-node, per-millisecond sequence component.
    pub const fn sequence(self) -> u16 {
        self.sequence
    }

    /// Recombines these components into the canonical 64-bit WebeID value.
    pub const fn to_id(self) -> WebeId {
        let raw = (self.time_millis << TIME_SHIFT)
            | ((self.node_id.value() as u64) << NODE_SHIFT)
            | (self.sequence as u64);
        WebeId::from_raw(raw)
    }

    pub(crate) const fn from_encoded_parts(time_millis: u64, node_id: u8, sequence: u16) -> Self {
        Self {
            time_millis,
            node_id: NodeId::from_u8(node_id),
            sequence,
        }
    }
}
