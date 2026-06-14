//! Incoming requests: request-line and header parsing, plus the body reader.

use std::collections::HashMap;
use std::pin::Pin;

use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::net::tcp::ReadHalf;

use crate::constants::{MAX_HEADERS_SIZE, MAX_REQUEST_LINE_SIZE};

/// A parsed client request within the supported HTTP/1.1 scope.
///
/// [`Request::new`] parses the request line (and validates the version);
/// [`Request::parse_headers`] reads the header block; the connection processor
/// then assigns [`Request::message_body`] based on the body framing.
pub struct Request<'r> {
    /// Accumulated parsed size, bounded by the request limits.
    pub total_size: usize,
    /// Uppercased HTTP method.
    pub method: String,
    /// Request target path.
    pub uri: String,
    /// HTTP version token; always `HTTP/1.1` for an accepted request.
    pub version: String,
    /// Lowercased header names mapped to comma-combined values.
    pub headers: Option<HashMap<String, String>>,
    /// The framed body reader, assigned by the connection processor.
    pub message_body: Option<Pin<Box<dyn AsyncBufRead + 'r + Send + Sync>>>,
}

/// Why a request could not be parsed or accepted.
#[derive(Debug)]
pub enum RequestError {
    /// An I/O error occurred reading from the stream.
    IOError(std::io::Error),
    /// The request line or a header was malformed or violated HTTP rules.
    MalformedRequestError,
    /// The request data could not be turned into anything meaningful.
    DeserializeError,
    /// The request line exceeded [`MAX_REQUEST_LINE_SIZE`].
    MaxURISizeError,
    /// The header block exceeded [`MAX_HEADERS_SIZE`].
    MaxHeaderSizeError,
    /// The request exceeded the maximum accepted size.
    MaxRequestSizeError,
    /// An unsupported transfer/content coding was requested.
    EncodingNotSupportedError,
    /// The request used an HTTP version other than `HTTP/1.1`. Holds the token.
    UnsupportedVersion(String),
}

impl From<std::io::Error> for RequestError {
    fn from(err: std::io::Error) -> RequestError {
        RequestError::IOError(err)
    }
}

impl<'r> Request<'r> {
    /// Parses the request line from `buf_reader` and validates the HTTP version.
    ///
    /// Returns [`RequestError::MalformedRequestError`] for a request line that is
    /// empty or does not have exactly three parts,
    /// [`RequestError::MaxURISizeError`] when the line exceeds
    /// [`MAX_REQUEST_LINE_SIZE`], and [`RequestError::UnsupportedVersion`] when
    /// the version is not `HTTP/1.1`.
    pub async fn new(
        buf_reader: &mut BufReader<ReadHalf<'_>>,
    ) -> Result<Request<'r>, RequestError> {
        // read in the first line and split it into method, target, and version
        let mut line = String::new();
        let mut line_reader = buf_reader.take(MAX_REQUEST_LINE_SIZE as u64);
        match line_reader.read_line(&mut line).await {
            Ok(0) => Err(RequestError::MalformedRequestError), // empty where a request line was expected
            Ok(uri_size) => {
                // read_line includes the ending newline; without it the line was
                // truncated by the size limit before the real end of the line.
                if !line.ends_with('\n') {
                    return Err(RequestError::MaxURISizeError);
                }
                let request_size: usize = uri_size;

                // parse request line: exactly METHOD SP target SP version
                let parts = line.splitn(3, ' ').collect::<Vec<&str>>();
                if parts.len() != 3 {
                    return Err(RequestError::MalformedRequestError);
                }
                let method = parts[0].to_uppercase().to_string();
                let uri = parts[1].to_string();
                let version = parts[2].trim().to_string();

                // enforce HTTP/1.1-only; any other version is rejected (505)
                if version != "HTTP/1.1" {
                    return Err(RequestError::UnsupportedVersion(version));
                }

                Ok(Request {
                    total_size: request_size,
                    method,
                    uri,
                    version,
                    headers: None,
                    message_body: None, // assigned later based on body framing
                })
            }
            // map the limit-reached signal to a header-size error
            Err(error) => match error.kind() {
                std::io::ErrorKind::NotFound => Err(RequestError::MaxHeaderSizeError),
                _ => Err(RequestError::IOError(error)),
            },
        }
    }

    /// Reads the header block, starting immediately after the request line.
    ///
    /// Header names are lowercased and duplicate names are comma-combined.
    /// Returns [`RequestError::MalformedRequestError`] for a header line missing
    /// its `:` separator and [`RequestError::MaxHeaderSizeError`] when the block
    /// exceeds [`MAX_HEADERS_SIZE`].
    pub async fn parse_headers(
        &mut self,
        buf_reader: &mut BufReader<ReadHalf<'_>>,
    ) -> Result<(), RequestError> {
        let parse_result = read_headers(buf_reader).await?;
        self.total_size += parse_result.1;
        self.headers = Some(parse_result.0);
        Ok(())
    }

    /// Assigns the framed body reader for this request.
    pub fn set_message_body(
        &mut self,
        message_body: Option<Pin<Box<dyn AsyncBufRead + 'r + Send + Sync>>>,
    ) {
        self.message_body = message_body;
    }
}

/// Reads and parses the header block, returning the headers and bytes consumed.
async fn read_headers(
    buf_reader: &mut BufReader<ReadHalf<'_>>,
) -> Result<(HashMap<String, String>, usize), RequestError> {
    let mut headers = HashMap::<String, String>::new();
    let reader = buf_reader.take(MAX_HEADERS_SIZE as u64);
    let mut lines = reader.lines();
    let mut terminated = false;
    // note: multiline headers were deprecated in RFC 7230, so we don't support them
    while let Some(line) = lines.next_line().await? {
        if line.is_empty() {
            terminated = true;
            break; // an empty line marks the end of the headers
        }
        let parts: Vec<&str> = line.splitn(2, ':').collect::<Vec<&str>>();
        if parts.len() == 2 {
            let field_name = parts[0].to_owned().to_lowercase();
            let field_value = parts[1].trim();
            match headers.get_mut(&field_name) {
                // RFC 2616: duplicate header names may be combined with commas
                Some(existing_value) => {
                    existing_value.push(',');
                    existing_value.push_str(field_value);
                }
                None => {
                    headers.insert(field_name, field_value.to_owned());
                }
            };
        } else {
            return Err(RequestError::MalformedRequestError);
        }
    }

    let remaining = lines.into_inner().limit() as usize;
    // the limit was exhausted before reaching the blank-line terminator
    if !terminated && remaining == 0 {
        return Err(RequestError::MaxHeaderSizeError);
    }

    let headers_size = MAX_HEADERS_SIZE - remaining;
    Ok((headers, headers_size))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    use tokio::net::{TcpListener, TcpStream};

    /// Returns a connected (client, server) TCP pair on the loopback interface.
    async fn connected() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let accept = async { listener.accept().await.unwrap().0 };
        let (client, server) = tokio::join!(TcpStream::connect(addr), accept);
        (client.unwrap(), server)
    }

    #[tokio::test]
    async fn request_line_over_limit_is_uri_size_error() {
        let (mut client, mut server) = connected().await;
        let writer = tokio::spawn(async move {
            // a request line with no terminating newline that exceeds the limit
            let mut line = b"GET /".to_vec();
            line.resize(MAX_REQUEST_LINE_SIZE + 100, b'a');
            let _ = client.write_all(&line).await;
            let _ = client.flush().await;
            // keep the socket open until the reader has finished
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        });

        let (read, _write) = server.split();
        let mut buf_reader = BufReader::new(read);
        let result = Request::new(&mut buf_reader).await;
        assert!(matches!(result, Err(RequestError::MaxURISizeError)));
        let _ = writer.await;
    }

    #[tokio::test]
    async fn header_block_over_limit_is_header_size_error() {
        let (mut client, mut server) = connected().await;
        let writer = tokio::spawn(async move {
            let mut data = b"GET / HTTP/1.1\r\n".to_vec();
            // valid header lines, never terminated by a blank line, exceeding the limit
            let mut header_line = b"hh: ".to_vec();
            header_line.resize(996, b'a');
            header_line.extend_from_slice(b"\r\n");
            while data.len() < MAX_HEADERS_SIZE + 2000 {
                data.extend_from_slice(&header_line);
            }
            let _ = client.write_all(&data).await;
            let _ = client.flush().await;
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        });

        let (read, _write) = server.split();
        let mut buf_reader = BufReader::new(read);
        let mut request = Request::new(&mut buf_reader)
            .await
            .expect("request line should parse");
        let result = request.parse_headers(&mut buf_reader).await;
        assert!(matches!(result, Err(RequestError::MaxHeaderSizeError)));
        let _ = writer.await;
    }
}
