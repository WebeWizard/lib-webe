use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;

use limit_read::LimitRead;

use crate::constants::{MAX_HEADER_SIZE, MAX_REQUEST_SIZE};

pub struct Request<'r> {
  pub size: usize,
  pub method: String,
  pub uri: String,
  pub headers: HashMap<String, String>,
  pub message_body: Option<Box<BufRead+'r>>,
}

pub enum RequestError {
  ReadError,           // error reading from stream
  MaxHeaderSizeError,  // individual header is too large
  MaxRequestSizeError, // request is too large
}

impl<'r> Request<'r> {
  pub fn new(buf_reader: &mut BufReader<&TcpStream>) -> Result<Request<'r>, RequestError> {
    //read in the first line.  Split it into Method and URI
    let mut line = String::new();
    match buf_reader.read_line_lim(&mut line, &MAX_HEADER_SIZE) {
      Ok(0) => return Err(RequestError::ReadError), // read an empty line when expecting request line
      Ok(uri_size) => {
        let mut request_size: usize = uri_size;
        let mut iter = line.split_whitespace();
        match iter.next() {
          // get method
          Some(method) => {
            match iter.next() {
              // get uri
              Some(uri) => {
                // get headers
                let mut headers = HashMap::<String, String>::new();
                let lines_iter = buf_reader.lines_lim(MAX_HEADER_SIZE);
                for line in lines_iter {
                  match line {
                    Ok(header) => {
                      request_size += header.len() + 2; // technically we don't know the size of the line terminator, assume \r\n
                      if request_size >= MAX_REQUEST_SIZE {
                        return Err(RequestError::MaxRequestSizeError);
                      }
                      if header == "".to_owned() {
                        break;
                      } else {
                        // TODO: assuming header is not split across multiple lines
                        // even though allowed by https://www.w3.org/Protocols/rfc2616/rfc2616-sec4.html#sec4.2
                        let mut header_iter = header.split(':');
                        match header_iter.next() {
                          // get header name
                          Some(header_name) => {
                            match header_iter.next() {
                              // get header value
                              Some(header_value) => {
                                headers.insert(
                                  header_name.to_string().to_lowercase(),
                                  header_value.trim().to_string(),
                                );
                              }
                              None => return Err(RequestError::ReadError), // expected header value
                            }
                          }
                          None => return Err(RequestError::ReadError), // expected header name
                        }
                      }
                    }
                    Err(error) => {
                      match error.kind() {
                        std::io::ErrorKind::NotFound => {
                          // header too large
                          return Err(RequestError::MaxHeaderSizeError);
                        }
                        _ => return Err(RequestError::ReadError),
                      }
                    }
                  }
                }
                return Ok(Request {
                  size: request_size,
                  method: method.to_uppercase().to_string(),
                  uri: uri.to_string(),
                  headers: headers,
                  message_body: None,
                });
                // TODO: if a 'Host' header is present, the URI is just an abs_path.
                // TODO: Do browsers provide the root '/' or is server expected to add it?
              }
              None => return Err(RequestError::ReadError), // expected non-whitespace
            }
          }
          None => return Err(RequestError::ReadError), // expected non-whitespace
        }
      }
      // catch max header/uri size
      Err(error) => {
        match error.kind() {
          std::io::ErrorKind::NotFound => {
            // header too large
            return Err(RequestError::MaxHeaderSizeError);
          }
          _ => return Err(RequestError::ReadError),
        }
      }
    }
  }

  pub fn set_message_body(&mut self, message_body: Option<Box<BufRead+'r>>) {
    self.message_body = message_body;
  }
}
