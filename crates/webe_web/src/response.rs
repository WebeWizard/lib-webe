//! Outgoing responses: status, headers, body, and on-the-wire framing.

use std::collections::HashMap;
use std::pin::Pin;

use tokio::io::{AsyncBufRead, AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::tcp::WriteHalf;

use super::status::Status;
use crate::body::{ResponseFraming, decide_response_framing};
use crate::constants::WEBE_BUFFER_SIZE;
use crate::encoding::chunked_encoder::encode_chunked;

/// A response: status, headers, an optional streamed body, and a connection
/// preference. Written to the client with explicit framing by [`Response::respond`].
pub struct Response {
    /// Status line code + reason.
    pub status: Status,
    /// Whether the connection may be kept alive after this response.
    pub keep_alive: bool,
    /// Response headers.
    pub headers: HashMap<String, String>,
    /// Optional streamed body reader.
    pub message_body: Option<Pin<Box<dyn AsyncBufRead + Send>>>,
}

/// Why writing a response failed.
#[derive(Debug)]
pub enum ResponseError {
    /// Reading from the response body failed mid-write.
    ReadError,
    /// Writing to the client socket failed.
    WriteError,
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseError::ReadError => write!(f, "failed to read from the response body"),
            ResponseError::WriteError => write!(f, "failed to write the response to the client"),
        }
    }
}

impl Response {
    /// Creates an empty response from a standard status code.
    pub fn new(status: u16) -> Response {
        Response {
            status: Status::from_standard_code(status),
            keep_alive: true,
            headers: HashMap::<String, String>::new(),
            message_body: None,
        }
    }

    /// Creates an empty response from an explicit [`Status`].
    pub fn from_status(status: Status) -> Response {
        Response {
            status,
            keep_alive: true,
            headers: HashMap::<String, String>::new(),
            message_body: None,
        }
    }

    /// Writes the response to `buf_writer` with explicit body framing.
    ///
    /// A body with a `Content-Length` header is sent verbatim; a body without a
    /// known length is streamed with `Transfer-Encoding: chunked`; a bodyless
    /// response sends neither framing header. The `Connection` header is set from
    /// [`Response::keep_alive`]. Returns [`ResponseError::ReadError`] if the body
    /// reader fails and [`ResponseError::WriteError`] on a socket write failure.
    pub async fn respond(
        &mut self,
        buf_writer: &mut BufWriter<WriteHalf<'_>>,
    ) -> Result<(), ResponseError> {
        let framing = decide_response_framing(self.message_body.is_some(), &self.headers);

        // reconcile the Connection header from the keep-alive preference
        let connection = if self.keep_alive {
            "keep-alive"
        } else {
            "close"
        };
        self.headers
            .insert("Connection".to_owned(), connection.to_owned());

        // a chunked body must not also carry a Content-Length
        if let ResponseFraming::Chunked = framing {
            self.headers
                .retain(|key, _| !key.eq_ignore_ascii_case("content-length"));
            self.headers
                .insert("Transfer-Encoding".to_owned(), "chunked".to_owned());
        }

        // write the status line
        let status_line = format!("HTTP/1.1 {} {}\r\n", self.status.code, self.status.reason);
        if buf_writer.write_all(status_line.as_bytes()).await.is_err() {
            return Err(ResponseError::WriteError);
        }
        // write the headers
        for (key, val) in self.headers.iter() {
            let header = format!("{key}: {val}\r\n");
            if buf_writer.write_all(header.as_bytes()).await.is_err() {
                return Err(ResponseError::WriteError);
            }
        }
        // blank line terminates the header block
        if buf_writer.write_all(b"\r\n").await.is_err() {
            return Err(ResponseError::WriteError);
        }

        // write the body according to the chosen framing
        match (framing, &mut self.message_body) {
            (ResponseFraming::None, _) | (_, None) => {}
            (ResponseFraming::Length, Some(body_reader)) => {
                let mut buf = [0u8; WEBE_BUFFER_SIZE];
                loop {
                    match body_reader.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(size) => {
                            if buf_writer.write_all(&buf[0..size]).await.is_err() {
                                return Err(ResponseError::WriteError);
                            }
                        }
                        Err(_error) => return Err(ResponseError::ReadError),
                    }
                }
            }
            (ResponseFraming::Chunked, Some(body_reader)) => {
                if encode_chunked(body_reader, buf_writer).await.is_err() {
                    return Err(ResponseError::WriteError);
                }
            }
        }

        // flush the stream
        match buf_writer.flush().await {
            Ok(_) => Ok(()),
            Err(_error) => Err(ResponseError::WriteError),
        }
    }
}
