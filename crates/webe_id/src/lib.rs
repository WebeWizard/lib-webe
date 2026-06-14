//! Compact, sortable 64-bit WebeID generation.
//!
//! WebeIDs are 64-bit values split into 40 bits of milliseconds since a custom
//! epoch, 8 bits of node identity, and 16 bits of per-node sequence.
//!
//! # Examples
//!
//! Generate and decompose an ID:
//!
//! ```
//! use webe_id::{Generator, NodeId};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut generator = Generator::builder(NodeId::from_u8(7)).build()?;
//! let id = generator.generate()?;
//! let components = id.components();
//! assert_eq!(components.node_id(), NodeId::from_u8(7));
//! # Ok(())
//! # }
//! ```
//!
//! Share a generator with ordinary synchronization when multiple workers need IDs:
//!
//! ```
//! use std::sync::{Arc, Mutex};
//! use std::thread;
//! use webe_id::{Generator, NodeId};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let generator = Generator::builder(NodeId::from_u8(3)).build()?;
//! let shared = Arc::new(Mutex::new(generator));
//!
//! let worker = {
//!     let shared = Arc::clone(&shared);
//!     thread::spawn(move || shared.lock().unwrap().generate().unwrap())
//! };
//!
//! let id = worker.join().map_err(|_| "worker panicked")?;
//! assert_eq!(id.components().node_id(), NodeId::from_u8(3));
//! # Ok(())
//! # }
//! ```
//!
//! Match typed failures directly:
//!
//! ```
//! use webe_id::{NodeId, NodeIdError};
//!
//! let error = NodeId::new(300).unwrap_err();
//! assert!(matches!(error, NodeIdError::OutOfRange { value: 300 }));
//! ```
//!
//! Preserve restart safety by persisting the full last generated WebeID and using
//! it as a marker when constructing the next generator:
//!
//! ```
//! use std::time::Duration;
//! use webe_id::time::default_epoch;
//! use webe_id::{Generator, NodeId, WebeIdComponents};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let epoch = default_epoch();
//! let last_id = WebeIdComponents::new(5, NodeId::from_u8(1), 10)?.to_id();
//! let mut generator = Generator::builder(NodeId::from_u8(1))
//!     .with_epoch(epoch)
//!     .with_restart_marker(last_id)
//!     .build()?;
//! let _next_id = generator.generate()?;
//! # let _ = epoch + Duration::from_millis(1);
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]
#![forbid(unsafe_code)]

/// WebeID component decomposition and recomposition.
pub mod components;
/// Typed WebeID errors.
pub mod error;
/// Stateful WebeID generation.
pub mod generator;
/// Canonical WebeID value type and representations.
pub mod id;
/// Node identifier validation.
pub mod node;
/// Epoch and clock utilities.
pub mod time;

/// Tokio-friendly bounded backpressure generation.
#[cfg(feature = "tokio")]
pub mod async_backpressure;

pub use components::WebeIdComponents;
pub use error::{
    BuildGeneratorError, GenerateError, NodeIdError, ParseWebeIdError, WebeIdComponentsError,
};
pub use generator::{Generator, GeneratorBuilder};
pub use id::{
    MAX_NODE_ID, MAX_SEQUENCE, MAX_TIME_MILLISECONDS, NODE_BITS, SEQUENCE_BITS, TIME_BITS, WebeId,
};
pub use node::NodeId;
