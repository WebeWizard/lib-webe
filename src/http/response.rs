use std::io::{BufRead, BufWriter, Write};
use std::collections::HashMap;
use std::net::TcpStream;

use super::request::{Request, RequestError};
use super::status::Status;

pub struct Response {
    status: Status,
    headers: HashMap<String, String>,
    message_body: Option<Box<BufRead>>
}

pub enum ResponseError {
    WriteError
}

impl Response {
    pub fn new(request: &Result<Request,RequestError>) -> Result<Response,ResponseError> {
        let headers = HashMap::<String,String>::new();
        match request {
            Ok(request) => {
                Ok(Response{status: Status::from_code(200).unwrap(), headers: headers, message_body: None})
            },
            Err(error) => {
                Ok(Response{status: Status::from_code(500).unwrap(), headers: headers, message_body: None})
            }
        }
    }

    pub fn respond(&self, mut buf_writer: BufWriter<&TcpStream>) -> Result<(),ResponseError> {
        // write the status line
        let status_line = format!("HTTP/1.1 {} {}\r\n", self.status.code, self.status.reason);
        match buf_writer.write_all(status_line.as_bytes()) {
            Ok(_) => {
                // write the response headers

            },
            Err(error) => return Err(ResponseError::WriteError)
        }
        // write the message body
        // flush the stream
        return Ok(());
    }
}