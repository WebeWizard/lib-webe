//! Responder validation types.
//!
//! [`Validation`] is an opaque, optional value threaded through responders so a
//! wrapping responder (e.g. an auth gate) can pass context to the inner one.

use super::status::Status;
use std::any::Any;

/// Optional context passed from one responder to the next during validation.
pub type Validation = Option<Box<dyn Any + Send>>;

/// The outcome of [`crate::responders::Responder::validate`]: either the
/// forwarded [`Validation`] or a [`Status`] to short-circuit with.
pub type ValidationResult = Result<Validation, Status>;
