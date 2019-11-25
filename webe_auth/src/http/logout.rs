use crate::AuthManager;
use crate::WebeAuth;
use webe_web::request::Request;
use webe_web::responders::Responder;
use webe_web::response::Response;
use webe_web::validation::Validation;

use std::collections::HashMap;

use serde::Deserialize;
use serde_json;

#[derive(Deserialize)]
pub struct LogoutForm {
  pub token: String,
}

pub struct LogoutResponder<'w> {
  auth_manager: &'w WebeAuth<'w>,
}

impl<'w> LogoutResponder<'w> {
  pub fn new(auth_manager: &'w WebeAuth) -> LogoutResponder<'w> {
    LogoutResponder {
      auth_manager: auth_manager,
    }
  }
}

impl<'w> Responder for LogoutResponder<'w> {
  // ALWAYS RETURN Ok(200) or Err(401) TO PREVENT LEAKING API INFORMATION
  fn build_response(
    &self,
    request: &mut Request,
    _params: &HashMap<String, String>,
    _validation: Validation,
  ) -> Result<Response, u16> {
    match &mut request.message_body {
      Some(body_reader) => {
        match serde_json::from_reader::<_, LogoutForm>(body_reader) {
          Ok(form) => {
            match self.auth_manager.logout(&form.token) {
              Ok(_) => {
                let response = Response::new(200);
                return Ok(response);
              }
              Err(_error) => {
                dbg!(_error); /* fall down into 401 response */
              }
            }
          }
          Err(_error) => {
            dbg!(_error); /* fall down into 401 response */
          }
        }
      }
      None => { /* fall down into 401 response */ }
    }
    // TODO: Have common response code responses be constants
    return Err(401);
  }
}
