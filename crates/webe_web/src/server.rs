//! The server lifecycle: bind, accept, and start.
//!
//! [`Server`] binds a TCP listener and, on [`Server::start`], accepts connections
//! and hands each one to the per-connection [`crate::processor`] loop on its own
//! task. Routing types live in [`crate::route`] and are re-exported here for
//! source compatibility.

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use tokio::net::TcpListener;

use crate::error::WebError;
use crate::processor::process_connection;
use crate::request::RequestError;
use crate::response::ResponseError;

// Re-export the routing types from their new home so existing
// `webe_web::server::{Route, RouteMap}` imports keep working.
pub use crate::route::{Route, RouteMap};

/// A bound HTTP server.
pub struct Server {
    /// The configured bind address.
    pub ip: Ipv4Addr,
    /// The configured bind port (`0` requests an OS-assigned port).
    pub port: u16,
    listener: TcpListener,
}

/// Legacy server failure type, retained for source compatibility.
///
/// New code should prefer [`WebError`]; `ServerError` converts into it via
/// `From<ServerError>`.
#[derive(Debug)]
pub enum ServerError {
    /// A request could not be processed.
    BadRequest(RequestError),
    /// The server failed to bind to the configured address/port.
    BindError(std::io::Error),
    /// The server failed to accept a connection.
    ConnectionFailed(std::io::Error),
    /// Processing the connection failed while writing the response.
    InternalError(ResponseError),
}

impl From<RequestError> for ServerError {
    fn from(err: RequestError) -> ServerError {
        ServerError::BadRequest(err)
    }
}

impl From<ResponseError> for ServerError {
    fn from(err: ResponseError) -> ServerError {
        ServerError::InternalError(err)
    }
}

impl Server {
    /// Binds a server to `ip`:`port`.
    ///
    /// Returns [`WebError::Bind`] (not a panic) when the address/port cannot be
    /// bound, for example because it is already in use. Pass port `0` to let the
    /// OS assign an ephemeral port, then read it back with [`Server::local_addr`].
    pub async fn new(ip: &Ipv4Addr, port: &u16) -> Result<Server, WebError> {
        match TcpListener::bind((*ip, *port)).await {
            Ok(listener) => Ok(Server {
                ip: *ip,
                port: *port,
                listener,
            }),
            Err(error) => Err(WebError::Bind(error)),
        }
    }

    /// Returns the actual local address the server is bound to.
    ///
    /// Useful when binding with port `0` to discover the OS-assigned port.
    pub fn local_addr(&self) -> Result<SocketAddr, WebError> {
        self.listener.local_addr().map_err(WebError::Bind)
    }

    /// Runs the accept loop, spawning a task per connection.
    ///
    /// Blocks the current task while the server runs. Returns
    /// [`WebError::Accept`] if accepting a connection fails. Per-connection
    /// failures are isolated to their own task and never stop the server.
    pub async fn start(&self, routes: RouteMap<'static>) -> Result<(), WebError> {
        let routes_arc = Arc::new(routes);
        loop {
            match self.listener.accept().await {
                Ok((stream, _socket)) => {
                    let process_routes = routes_arc.clone();
                    tokio::spawn(async move {
                        let _ = process_connection(stream, process_routes).await;
                    });
                }
                Err(error) => return Err(WebError::Accept(error)),
            }
        }
    }
}
