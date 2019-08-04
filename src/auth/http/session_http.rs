use crate::auth::WebeAuth;
use crate::http::request::Request;
use crate::http::responders::Responder;
use crate::http::response::Response;

use std::collections::HashMap;
use std::io::Cursor;

pub struct LoginResponder<'w> {
  auth_manager: &'w WebeAuth,
}

impl<'w> LoginResponder<'w> {
  pub fn new(auth_manager: &'w WebeAuth) -> LoginResponder<'w> {
    LoginResponder {
      auth_manager: auth_manager,
    }
  }
}

impl<'w> Responder for LoginResponder<'w> {
  fn validate(&self, _request: &Request, params: &HashMap<String, String>) -> Result<u16, u16> {
    // TODO: make sure email and password are present in request
    dbg!("LoginResponder validating");
    return Ok(200);
  }

  fn build_response(
    &self,
    _request: &Request,
    _params: &HashMap<String, String>,
    validation_code: u16,
  ) -> Result<Response, u16> {
    dbg!("LoginResponder building response");
    if validation_code == 200 {
      let email = "WebeWizardSessionTest@gmail.com".to_owned();
      let pass = "test123".to_owned();
      match self.auth_manager.login(&email, &pass) {
        Ok(session) => {
          let message = "Ok".to_owned();
          let bytes = message.clone().into_bytes();
          let mut headers = HashMap::<String, String>::new();
          headers.insert("Content-Length".to_owned(), bytes.len().to_string());
          headers.insert("Content-Type".to_owned(), "text/html".to_owned());
          let mut response = Response::new(validation_code);
          response.headers = headers;
          response.message_body = Some(Box::new(Cursor::new(bytes)));
          return Ok(response);
        }
        Err(error) => {
          let message = "Auth failed".to_owned();
          let bytes = message.clone().into_bytes();
          let mut headers = HashMap::<String, String>::new();
          headers.insert("Content-Length".to_owned(), bytes.len().to_string());
          headers.insert("Content-Type".to_owned(), "text/html".to_owned());
          let mut response = Response::new(401);
          response.headers = headers;
          response.message_body = Some(Box::new(Cursor::new(bytes)));
          return Ok(response);
        }
      }
    } else {
      let response = Response::new(400);
      return Ok(response);
    }
  }
}
