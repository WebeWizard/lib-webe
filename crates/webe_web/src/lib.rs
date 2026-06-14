//! `webe_web` is a small, async HTTP/1.1 server library.
//!
//! It is **not** a feature-complete HTTP implementation. It commits to a
//! documented subset of HTTP/1.1 (see the crate `README.md`) and is reachable
//! through the `webe::web` facade. The crate is organized into focused modules:
//!
//! - [`server`] — bind / accept / start lifecycle ([`server::Server`]).
//! - [`route`] — [`route::Route`], [`route::RouteMap`], and deterministic matching.
//! - [`processor`] — the per-connection request lifecycle.
//! - [`request`] / [`response`] — request parsing and framed response writing.
//! - [`body`] — request and response body-framing decisions.
//! - [`error`] — the consolidated, categorized [`error::WebError`].
//! - [`responders`] — the [`responders::Responder`] trait and built-in responders.
#![deny(missing_docs)]
pub mod body;
pub mod constants;
pub mod encoding;
pub mod error;
pub mod processor;
pub mod request;
pub mod responders;
pub mod response;
pub mod route;
pub mod server;
pub mod status;
pub mod validation;
