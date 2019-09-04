use crate::auth::WebeAuth;
use crate::http::request::Request;
use crate::http::responders::static_message::StaticResponder;
use crate::http::responders::Responder;
use crate::http::response::Response;

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginForm {
  pub email: String,
  pub pass: String,
}

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
  fn validate(&self, _request: &Request, _params: &HashMap<String, String>) -> Result<u16, u16> {
    // TODO: make sure email and password are present in request
    dbg!("LoginResponder validating");
    return Ok(200);
  }

  // ALWAYS RETURN Ok(200) or Err(401) TO PREVENT LEAKING API INFORMATION
  fn build_response(
    &self,
    request: &mut Request,
    params: &HashMap<String, String>,
    validation_code: u16,
  ) -> Result<Response, u16> {
    if validation_code != 200 {
      /* fall down into 401 response */
    } else {
      match &mut request.message_body {
        Some(body_reader) => {
          match serde_json::from_reader::<_, LoginForm>(body_reader) {
            Ok(form) => {
              match self.auth_manager.login(&form.email, &form.pass) {
                Ok(session) => {
                  match serde_json::to_string(&session) {
                    Ok(message) => {
                      let responder = StaticResponder::new(200, message);
                      return responder.build_response(request, params, 200);
                    }
                    Err(_error) => { /* fall down into 401 response */ }
                  }
                }
                Err(_error) => { /* fall down into 401 response */ }
              }
            }
            Err(_error) => { /* fall down into 401 response */ }
          }
        }
        None => { /* fall down into 401 response */ }
      }
    }
    // TODO: Have common response code responses be constants
    let static_responder = StaticResponder::from_standard_code(401);
    return static_responder.build_response(request, params, 401);
  }
}
