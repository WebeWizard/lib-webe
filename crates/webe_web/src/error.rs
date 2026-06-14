//! The consolidated, categorized crate error type, [`WebError`].
//!
//! Failures across the server, request parsing, body framing, routing, response
//! writing, and responders are unified into a single matchable taxonomy. Each
//! category's [`std::fmt::Display`] names what failed, and [`WebError::client_status`]
//! maps client-visible failures to the documented HTTP status code.

use crate::body::BodyError;
use crate::request::RequestError;
use crate::response::ResponseError;
use crate::route::RoutingError;
use crate::server::ServerError;

/// A categorized web-server failure.
#[derive(Debug)]
pub enum WebError {
    /// Failed to bind the listener (server setup; not client-visible).
    Bind(std::io::Error),
    /// Failed to accept a connection (server failure; not client-visible).
    Accept(std::io::Error),
    /// The request line or headers were malformed or exceeded a limit (`400`).
    Request(RequestError),
    /// The request used an unsupported HTTP version (`505`). Holds the version.
    Version(String),
    /// The request or response body could not be framed (`400`).
    Body(BodyError),
    /// The request could not be routed (`404` / `405`).
    Routing(RoutingError),
    /// Writing the response failed, or a body reader failed mid-write.
    Response(ResponseError),
    /// A responder rejected the request; holds the responder-provided status.
    Responder(u16),
}

impl WebError {
    /// The HTTP status code to send to the client for this failure, or `None`
    /// for server-side failures (bind/accept) that never reach a client.
    pub fn client_status(&self) -> Option<u16> {
        match self {
            WebError::Bind(_) | WebError::Accept(_) => None,
            WebError::Request(_) => Some(400),
            WebError::Version(_) => Some(505),
            WebError::Body(_) => Some(400),
            WebError::Routing(RoutingError::NotFound) => Some(404),
            WebError::Routing(RoutingError::MethodNotAllowed) => Some(405),
            WebError::Response(_) => Some(500),
            WebError::Responder(code) => Some(*code),
        }
    }
}

impl std::fmt::Display for WebError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebError::Bind(e) => {
                write!(f, "bind: could not bind the server socket: {e}")
            }
            WebError::Accept(e) => {
                write!(f, "accept: could not accept a connection: {e}")
            }
            WebError::Request(e) => write!(f, "request: {e:?} (400)"),
            WebError::Version(v) => write!(
                f,
                "version: unsupported HTTP version '{v}'; only HTTP/1.1 is supported (505)"
            ),
            WebError::Body(e) => write!(f, "{e}"),
            WebError::Routing(e) => write!(f, "{e}"),
            WebError::Response(e) => write!(f, "response: {e}"),
            WebError::Responder(code) => {
                write!(f, "responder: rejected the request with status {code}")
            }
        }
    }
}

impl std::error::Error for WebError {}

impl From<RequestError> for WebError {
    fn from(err: RequestError) -> WebError {
        match err {
            RequestError::UnsupportedVersion(version) => WebError::Version(version),
            other => WebError::Request(other),
        }
    }
}

impl From<BodyError> for WebError {
    fn from(err: BodyError) -> WebError {
        WebError::Body(err)
    }
}

impl From<RoutingError> for WebError {
    fn from(err: RoutingError) -> WebError {
        WebError::Routing(err)
    }
}

impl From<ResponseError> for WebError {
    fn from(err: ResponseError) -> WebError {
        WebError::Response(err)
    }
}

impl From<ServerError> for WebError {
    fn from(err: ServerError) -> WebError {
        match err {
            ServerError::BadRequest(e) => WebError::from(e),
            ServerError::BindError(e) => WebError::Bind(e),
            ServerError::ConnectionFailed(e) => WebError::Accept(e),
            ServerError::InternalError(e) => WebError::Response(e),
        }
    }
}
