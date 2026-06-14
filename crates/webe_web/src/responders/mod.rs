//! The [`Responder`] trait and the crate's built-in responders.

/// File-serving responder.
pub mod file;
/// `OPTIONS` preflight responder.
pub mod options;
/// Single-page-application fallback responder.
pub mod spa;
/// Fixed status + message responder.
pub mod static_message;

use async_trait::async_trait;

use super::request::Request;
use super::response::Response;
use super::status::Status;
use super::validation::Validation;
use super::validation::ValidationResult;

/// A handler that validates and responds to a routed request.
///
/// Implementors decide whether a request is worth answering ([`Responder::validate`])
/// and produce the [`Response`] ([`Responder::build_response`]). Captured route
/// parameters are delivered as `&Vec<(String, String)>` in route-declaration order
/// (see the public API contract).
#[async_trait]
pub trait Responder: Send + Sync {
    /// Decides whether the request should be answered.
    ///
    /// Returns `Ok(validation)` to forward the (possibly updated) [`Validation`]
    /// to [`Responder::build_response`], or `Err(status_code)` to short-circuit
    /// with a status. The default implementation forwards the validation
    /// unchanged, which supports wrapping responders (e.g. "secure"/"logged-in").
    // The `&Vec` parameter is mandated by the public API contract; see module docs.
    #[allow(clippy::ptr_arg)]
    async fn validate(
        &self,
        _request: &Request,
        _params: &Vec<(String, String)>,
        validation: Validation,
    ) -> ValidationResult {
        Ok(validation) // default is to forward the validation along
    }

    /// Produces the response for a validated request.
    ///
    /// `request` is mutable so the body reader can be consumed. `params` carries
    /// the captured route parameters. Returns the [`Response`], or a fallback
    /// status code that the connection processor renders as a static error.
    // The `&Vec` parameter is mandated by the public API contract; see module docs.
    #[allow(clippy::ptr_arg)]
    async fn build_response(
        &self,
        request: &mut Request,
        params: &Vec<(String, String)>,
        validation: Validation,
    ) -> Result<Response, u16>;
}
