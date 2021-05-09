use std::boxed::Box;
use std::collections::HashMap;
use std::io::Cursor;

use super::Request;
use super::Responder;
use super::Response;
use super::Status;
use super::Validation;

pub struct StaticResponder {
  status_code: u16,
  message: String, // TODO: take a reference instead of owning it
}

impl StaticResponder {
  pub fn new(status_code: u16, message: String) -> StaticResponder {
    StaticResponder {
      status_code: status_code,
      message: message,
    }
  }

  pub fn from_status(status: Status) -> StaticResponder {
    StaticResponder {
      status_code: status.code,
      message: status.reason,
    }
  }

  pub fn from_standard_code(status_code: u16) -> StaticResponder {
    let status = Status::from_standard_code(status_code);
    return StaticResponder::from_status(status);
  }

  // since this responder doesn't really need the request, params, or validation
  // we can prepare responses much easier
  pub fn quick_response<'r>(&self) -> Response<'r> {
    let bytes = self.message.clone().into_bytes();
    let mut headers = HashMap::<String, String>::new();
    headers.insert("Content-Length".to_owned(), bytes.len().to_string());
    headers.insert("Content-Type".to_owned(), "text/html".to_owned());
    let mut response = Response::new(self.status_code);
    response.headers = headers;
    response.message_body = Some(Box::pin(Cursor::new(bytes)));
    return response;
  }
}

impl Responder for StaticResponder {
  fn build_response(
    &self,
    _request: &mut Request,
    _params: &Vec<(String, String)>,
    _validation: Validation,
  ) -> Result<Response, u16> {
    Ok(self.quick_response())
  }
}
