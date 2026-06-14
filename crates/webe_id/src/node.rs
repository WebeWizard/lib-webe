//! Node identifier validation helpers.

use std::fmt;

use crate::error::NodeIdError;

/// A validated 8-bit node identifier encoded into each WebeID.
///
/// Node values `0` and `255` are valid. Callers are responsible for assigning
/// distinct node IDs to concurrently active generators in the same uniqueness
/// domain.
///
/// # Examples
///
/// ```
/// use webe_id::{NodeId, NodeIdError};
///
/// assert_eq!(NodeId::new(255)?.value(), 255);
/// assert!(matches!(
///     NodeId::new(256),
///     Err(NodeIdError::OutOfRange { value: 256 })
/// ));
/// # Ok::<(), NodeIdError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId(u8);

impl NodeId {
    /// Creates a node ID from a configuration value in the supported `0..=255` range.
    pub fn new(value: u16) -> Result<Self, NodeIdError> {
        let node = u8::try_from(value).map_err(|_| NodeIdError::OutOfRange { value })?;
        Ok(Self(node))
    }

    /// Creates a node ID from an already bounded byte value.
    pub const fn from_u8(value: u8) -> Self {
        Self(value)
    }

    /// Returns the numeric node value.
    pub const fn value(self) -> u8 {
        self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl From<NodeId> for u8 {
    fn from(node_id: NodeId) -> Self {
        node_id.value()
    }
}

impl TryFrom<u16> for NodeId {
    type Error = NodeIdError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
