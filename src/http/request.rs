use std::collections::HashMap;
use std::net::TcpStream;
use std::io::{BufRead, BufReader};

pub struct Request {
	pub method: String,
	pub uri: String,
	pub headers: HashMap<String, String>
}

pub enum RequestError {
    ReadError // error reading from stream
}

impl Request {
    pub fn new (mut buf_reader: BufReader<&TcpStream>) -> Result<Request,RequestError> {
        //read in the first line.  Split it into Method and URI
		let mut line = String::new();
		match buf_reader.read_line(&mut line) {
            Ok(0) => return Err(RequestError::ReadError), // read an empty line when expecting request line
            Ok(_) => {
                let mut iter = line.split_whitespace();
                match iter.next() { // get method
                    Some(method) => {
                        match iter.next() { // get uri
                            Some(uri) => {
                                // get headers
                                let mut headers = HashMap::<String,String>::new();
                                loop {
                                    let mut header = String::new();
                                    match buf_reader.read_line(&mut header) {
                                        Ok(0) => break, // empty line indicates end of headers
                                        Ok(_) => {
                                            // TODO: assuming header is not split across multiple lines
                                            // even though allowed by https://www.w3.org/Protocols/rfc2616/rfc2616-sec4.html#sec4.2
                                            let mut header_iter = header.split(':');
                                            match header_iter.next() { // get header name
                                                Some(header_name) => {
                                                    match header_iter.next() { // get header value
                                                        Some(header_value) => {
                                                            headers.insert(header_name.to_string(), header_value.trim().to_string());
                                                        },
                                                        None => return Err(RequestError::ReadError) // expected header value
                                                    }
                                                },
                                                None => return Err(RequestError::ReadError) // expected header name
                                            }
                                        },
                                        Err(_) => return Err(RequestError::ReadError)
                                    }
                                }
                                return Ok( Request{ 
                                    method: method.to_string(),
                                    uri: uri.to_string(),
                                    headers: headers
                                } );
                                // TODO: if a 'Host' header is present, the URI is just an abs_path.
                                // TODO: Do browsers provide the root '/' or is server expected to add it?
                            },
                            None => return Err(RequestError::ReadError) // expected non-whitespace
                        }
                    },
                    None => return Err(RequestError::ReadError) // expected non-whitespace
                }
            },
            Err(_) => return Err(RequestError::ReadError)
        }
    }
}