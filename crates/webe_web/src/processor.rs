//! The per-connection request lifecycle.
//!
//! [`process_connection`] owns an accepted stream, splits it into a buffered
//! reader/writer pair, and runs the keep-alive loop: parse a request, route it,
//! frame its body, invoke the responder, and write a framed response. Every
//! recognized failure is mapped to a documented static error response so the
//! connection task ends cleanly without ever stopping the server.

use std::pin::Pin;
use std::sync::Arc;

use tokio::io::{AsyncBufRead, AsyncReadExt, BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::net::tcp::ReadHalf;

use crate::body::{RequestBody, decide_request_body};
use crate::encoding::chunked::ChunkedDecoder;
use crate::error::WebError;
use crate::request::Request;
use crate::responders::static_message::StaticResponder;
use crate::response::Response;
use crate::route::{RoutingError, parse_route_params};
use crate::server::RouteMap;

/// Runs the request lifecycle for a single accepted connection.
///
/// Parses requests in a keep-alive loop until the connection should close.
/// Recognized request, routing, body, and responder failures are turned into
/// the documented static error responses (`400`/`404`/`405`/`505`/responder
/// status) and the connection is closed afterward. Returns [`WebError`] only for
/// an unrecoverable socket write failure.
pub async fn process_connection(
    mut stream: TcpStream,
    routes: Arc<RouteMap<'_>>,
) -> Result<(), WebError> {
    let (reader, writer) = stream.split();
    let mut buf_reader = BufReader::new(reader);
    let mut buf_writer = BufWriter::new(writer);

    let mut keep_alive = true;
    while keep_alive {
        let mut response = match build_response(&mut buf_reader, &routes).await {
            Ok((response, alive)) => {
                keep_alive = alive;
                response
            }
            Err(code) => {
                // Any recognized failure closes the connection after replying.
                keep_alive = false;
                StaticResponder::from_standard_code(code).quick_response()
            }
        };
        response.keep_alive = keep_alive;
        response.respond(&mut buf_writer).await?;
    }

    Ok(())
}

/// Parses, routes, frames, and dispatches a single request.
///
/// On success returns the responder's [`Response`] and whether the connection
/// may be kept alive. On any recognized failure returns the documented client
/// status code (`Err(code)`), which the caller renders as a static response.
async fn build_response(
    buf_reader: &mut BufReader<ReadHalf<'_>>,
    routes: &RouteMap<'_>,
) -> Result<(Response, bool), u16> {
    // --- request line + version ---
    let mut request = match Request::new(buf_reader).await {
        Ok(request) => request,
        Err(error) => return Err(status_for(error)),
    };

    // --- routing (404 vs 405) ---
    let route = match routes.find_best_route(&request) {
        Ok(route) => route,
        Err(RoutingError::NotFound) => return Err(404),
        Err(RoutingError::MethodNotAllowed) => return Err(405),
    };
    let responder = match routes.responder_for(route) {
        Some(responder) => responder,
        None => return Err(500), // unreachable: route came from this map
    };
    let params = parse_route_params(&request, route);

    // --- headers ---
    if let Err(error) = request.parse_headers(buf_reader).await {
        return Err(status_for(error));
    }

    // --- request body framing ---
    let framing = match decide_request_body(request.headers.as_ref()) {
        Ok(framing) => framing,
        Err(_body_error) => return Err(400),
    };
    let mut body_reader: Pin<Box<dyn AsyncBufRead + Send + Sync>> = Box::pin(&mut *buf_reader);
    match framing {
        RequestBody::None => {}
        RequestBody::Length(length) => {
            body_reader = Box::pin(body_reader.take(length));
        }
        RequestBody::Chunked => {
            body_reader = Box::pin(BufReader::new(ChunkedDecoder::new(body_reader)));
        }
    }

    // --- keep-alive intent from the request ---
    let mut keep_alive = true;
    if let Some(headers) = &request.headers
        && let Some(connection) = headers.get("connection")
        && connection.to_lowercase().contains("close")
    {
        keep_alive = false;
    }

    request.set_message_body(Some(body_reader));

    // --- validate + build ---
    match responder.validate(&request, &params, None).await {
        Ok(validation) => match responder
            .build_response(&mut request, &params, validation)
            .await
        {
            Ok(response) => Ok((response, keep_alive)),
            Err(code) => Err(code),
        },
        Err(status) => Err(status.code),
    }
}

/// Maps a parsing failure to the documented client status code.
fn status_for(error: crate::request::RequestError) -> u16 {
    WebError::from(error).client_status().unwrap_or(400)
}
