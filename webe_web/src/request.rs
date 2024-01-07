use std::collections::HashMap;
use std::pin::Pin;

use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::net::tcp::ReadHalf;

use crate::constants::{MAX_HEADERS_SIZE, MAX_REQUEST_LINE_SIZE};

pub struct Request<'r> {
    pub total_size: usize,
    pub method: String,
    pub uri: String,
    pub version: String,
    pub headers: Option<HashMap<String, String>>,
    pub message_body: Option<Pin<Box<dyn AsyncBufRead + 'r + Send + Sync>>>,
}

#[derive(Debug)]
pub enum RequestError {
    IOError(std::io::Error), // io error reading from stream
    MalformedRequestError, // generic error for un-parseable requests and requests that don't meet http standards
    DeserializeError,      // failed to turn the request data into anything meaningful
    MaxURISizeError,       // Size of URI is too large
    MaxHeaderSizeError,    // Size of headers is too large
    MaxRequestSizeError,   // request is too large
    EncodingNotSupportedError,
}

impl From<std::io::Error> for RequestError {
    fn from(err: std::io::Error) -> RequestError {
        RequestError::IOError(err)
    }
}

impl<'r> Request<'r> {
    pub async fn new(
        buf_reader: &mut BufReader<ReadHalf<'_>>,
    ) -> Result<Request<'r>, RequestError> {
        //read in the first line.  Split it into Method and URI
        let mut line = String::new();
        let mut line_reader = buf_reader.take(MAX_REQUEST_LINE_SIZE as u64);
        match line_reader.read_line(&mut line).await {
            Ok(0) => return Err(RequestError::MalformedRequestError), // read an empty line when expecting request line
            Ok(uri_size) => {
                // read_line should contain ending new line char. otherwise we reached the end of Take without finding real end of URI
                if !line.ends_with('\n') {
                    return Err(RequestError::MaxURISizeError);
                }
                let request_size: usize = uri_size;

                // parse request line
                let parts = line.splitn(3, ' ').collect::<Vec<&str>>();
                if parts.len() != 3 {
                    return Err(RequestError::MalformedRequestError);
                } // request line should only have 3 parts
                let method = parts[0].to_uppercase().to_string(); // TODO: supported methods should probably be enumerated
                let uri = parts[1].to_string();
                let version = parts[2].to_string(); // TODO: supported versions should probably be enumerated

                return Ok(Request {
                    total_size: request_size,
                    method: method,
                    uri: uri,
                    version: version,
                    headers: None,
                    message_body: None, // the server will assign an appropriate reader based on the request type.
                });
            }
            // catch max header/uri size
            Err(error) => {
                match error.kind() {
                    std::io::ErrorKind::NotFound => {
                        // header too large
                        return Err(RequestError::MaxHeaderSizeError);
                    }
                    _ => return Err(RequestError::IOError(error)),
                }
            }
        }
    }

    // reads headers from the stream. this function expects to start reading from position immediately after parsing the request line
    pub async fn parse_headers(
        &mut self,
        buf_reader: &mut BufReader<ReadHalf<'_>>,
    ) -> Result<(), RequestError> {
        let parse_result = read_headers(buf_reader).await?;
        dbg!(&parse_result);
        self.total_size = self.total_size + parse_result.1;
        self.headers = Some(parse_result.0);
        Ok(())
    }

    pub fn set_message_body(
        &mut self,
        message_body: Option<Pin<Box<dyn AsyncBufRead + 'r + Send + Sync>>>,
    ) {
        self.message_body = message_body;
    }
}

async fn read_headers(
    buf_reader: &mut BufReader<ReadHalf<'_>>,
) -> Result<(HashMap<String, String>, usize), RequestError> {
    let mut headers = HashMap::<String, String>::new();
    let reader = buf_reader.take(MAX_HEADERS_SIZE as u64);
    let mut lines = reader.lines();
    // TODO: handle max header size error
    while let Some(line) = lines.next_line().await? {
        if line.is_empty() {
            break;
        } // an empty line marks the end of http headers
        let parts: Vec<&str> = line.splitn(2, ':').collect::<Vec<&str>>(); // note: multiline headers were deprecated in rfc7230 so we won't support them
        if parts.len() == 2 {
            let field_name = parts[0].to_owned();
            let field_value = parts[1].trim();
            // TODO: need to normalize capitalization?
            match headers.get_mut(&field_name) {
                // http 1.1 rfc2616 says multiple headers with identical names can be combined with commas
                Some(existing_value) => {
                    existing_value.push(',');
                    existing_value.push_str(field_value);
                }
                None => {
                    headers.insert(field_name, field_value.to_owned());
                    ()
                }
            };
        } else {
            return Err(RequestError::MalformedRequestError);
        }
    }

    let headers_size = lines.into_inner().limit() as usize;

    return Ok((headers, headers_size));
}
