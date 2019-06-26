use std::collections::HashMap;
use std::io::{BufRead, BufWriter, Write};
use std::net::TcpStream;

use super::status::Status;
use crate::constants::WEBE_BUFFER_SIZE;

pub struct Response {
  pub status: Status,
  pub keep_alive: bool, // flag to tell the server to keep alive after responding
  pub headers: HashMap<String, String>,
  pub message_body: Option<Box<BufRead>>,
}

pub enum ResponseError {
  ReadError, // error reading from message body
  WriteError,
}

impl Response {
  // create an empty response from a status code
  pub fn new(status: u16) -> Response {
    let headers = HashMap::<String, String>::new();
    return Response {
      status: Status::from_standard_code(status),
      keep_alive: true,
      headers: headers,
      message_body: None,
    };
  }

  pub fn from_status(status: Status) -> Response {
    let headers = HashMap::<String, String>::new();
    return Response {
      status: status,
      keep_alive: true,
      headers: headers,
      message_body: None,
    };
  }

  pub fn respond(&mut self, mut buf_writer: BufWriter<&TcpStream>) -> Result<(), ResponseError> {
    // write the status line
    let status_line = format!("HTTP/1.1 {} {}\r\n", self.status.code, self.status.reason);
    match buf_writer.write_all(status_line.as_bytes()) {
      Ok(_) => {
        // reconcile keep-alive header
        if self.keep_alive {
          self
            .headers
            .insert("Connection".to_owned(), "keep-alive".to_owned());
        }
        // write the response headers
        for (key, val) in self.headers.iter() {
          let header: String = format!("{}: {}\r\n", key, val);
          match buf_writer.write(header.as_bytes()) {
            Ok(_) => continue,
            Err(_error) => return Err(ResponseError::WriteError),
          }
        }
      }
      Err(_error) => return Err(ResponseError::WriteError),
    }
    // write the message body
    match &mut self.message_body {
      Some(body_reader) => {
        // begin with empty new line
        match buf_writer.write(b"\r\n") {
          Ok(_) => {
            // iterate through message_body until it's empty
            // TODO: does fiddling with the buffer size help performance?
            let mut buf = [0u8; WEBE_BUFFER_SIZE];
            loop {
              match body_reader.read(&mut buf) {
                Ok(0) => break,
                Ok(size) => match buf_writer.write(&buf[0..size]) {
                  Ok(_) => {}
                  Err(_error) => return Err(ResponseError::WriteError),
                },
                Err(_error) => return Err(ResponseError::ReadError),
              }
            }
          }
          Err(_error) => return Err(ResponseError::WriteError),
        }
      }
      None => {}
    }
    // flush the stream
    match buf_writer.flush() {
      Ok(_) => return Ok(()),
      Err(_error) => return Err(ResponseError::WriteError),
    }
  }
}
