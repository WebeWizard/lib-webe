use std::boxed::Box;
use std::collections::HashMap;
use std::io::Cursor;

use super::Status;
use super::Request;
use super::Responder;
use super::Response;

#[derive(Clone)]
pub struct StaticResponder {
    status_code: u16,
    message: String
}

impl StaticResponder {
    pub fn new (status_code: u16, message: String) -> StaticResponder {
        StaticResponder { status_code: status_code, message: message }
    }

    pub fn from_status (status: Status) -> StaticResponder {
        StaticResponder { status_code: status.code, message: status.reason }
    }
}

impl Responder for StaticResponder {
    fn validate(&self, request: &Request, params: &HashMap<String,String>) -> Result<u16,u16> {
        Ok(200)
    }

    fn build_response(&self, request: &Request, params: &HashMap<String,String>, validation_code: u16) -> Result<Response,u16> {
        let bytes = self.message.clone().into_bytes();
        let mut headers = HashMap::<String, String>::new();
        headers.insert("Content-Length".to_owned(), bytes.len().to_string());
        let mut response = Response::new(200);
        response.headers = headers;
        let bytes = self.message.clone().into_bytes();
        response.message_body = Some(Box::new(Cursor::new(bytes)));
        return Ok(response);
    }
}