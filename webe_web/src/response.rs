use std::collections::HashMap;
use std::pin::Pin;

use tokio::net::tcp::WriteHalf;
use tokio::io::{AsyncBufRead, AsyncReadExt, BufWriter, AsyncWriteExt};

use super::status::Status;
use crate::constants::WEBE_BUFFER_SIZE;

pub struct Response<'r> {
  pub status: Status,
  pub keep_alive: bool, // flag to tell the server to keep alive after responding
  pub headers: HashMap<String, String>,
  pub message_body: Option<Pin<Box<dyn AsyncBufRead + 'r + Send>>>,
}

pub enum ResponseError {
  ReadError, // error reading from message body
  WriteError,
}

impl<'r> Response<'r> {
  // create an empty response from a status code
  pub fn new(status: u16) -> Response<'r> {
    let headers = HashMap::<String, String>::new();
    return Response {
      status: Status::from_standard_code(status),
      keep_alive: true,
      headers: headers,
      message_body: None,
    };
  }

  pub fn from_status(status: Status) -> Response<'r> {
    let headers = HashMap::<String, String>::new();
    return Response {
      status: status,
      keep_alive: true,
      headers: headers,
      message_body: None,
    };
  }

  pub async fn respond(&mut self, buf_writer: &mut BufWriter<WriteHalf<'_>>) -> Result<(), ResponseError> {
    // write the status line
    let status_line = format!("HTTP/1.1 {} {}\r\n", self.status.code, self.status.reason);
    match buf_writer.write_all(status_line.as_bytes()).await {
      Ok(_) => {
        // reconcile keep-alive header
        if self.keep_alive {
          // TODO: this should be handled by the server process_stream, not the response
          self
            .headers
            .insert("Connection".to_owned(), "keep-alive".to_owned());
        }
        // write the response headers
        for (key, val) in self.headers.iter() {
          let header: String = format!("{}: {}\r\n", key, val);
          match buf_writer.write(header.as_bytes()).await {
            Ok(_) => continue,
            Err(_error) => return Err(ResponseError::WriteError),
          }
        }
      }
      Err(_error) => return Err(ResponseError::WriteError),
    }
    // begin with empty new line
    match buf_writer.write(b"\r\n").await {
      Ok(_) => {
        // write the message body
        match &mut self.message_body {
          Some(body_reader) => {
            // iterate through message_body until it's empty
            // TODO: does fiddling with the buffer size help performance?
            let mut buf = [0u8; WEBE_BUFFER_SIZE];
            loop {
              match body_reader.read(&mut buf).await {
                Ok(0) => break,
                Ok(size) => match buf_writer.write(&buf[0..size]).await {
                  Ok(_) => {}
                  Err(_error) => return Err(ResponseError::WriteError),
                },
                Err(_error) => return Err(ResponseError::ReadError),
              }
            }
          }
          None => {}
        }
      }
      Err(_error) => return Err(ResponseError::WriteError),
    }
    // flush the stream
    match buf_writer.flush().await {
      Ok(_) => return Ok(()),
      Err(_error) => return Err(ResponseError::WriteError),
    }
  }
}
