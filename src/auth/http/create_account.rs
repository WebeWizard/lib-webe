use crate::auth::WebeAuth;
use crate::http::request::Request;
use crate::http::responders::static_message::StaticResponder;
use crate::http::responders::Responder;
use crate::http::response::Response;

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateAccountForm {
    pub email: String,
    pub pass: String,
}

pub struct CreateAccountResponder<'w> {
  auth_manager: &'w WebeAuth,
}

impl<'w> CreateAccountResponder<'w> {
  pub fn new(auth_manager: &'w WebeAuth) -> CreateAccountResponder<'w> {
    CreateAccountResponder {
      auth_manager: auth_manager,
    }
  }
}

impl<'w> Responder for CreateAccountResponder<'w> {
  fn validate(&self, _request: &Request, _params: &HashMap<String, String>) -> Result<u16, u16> {
    // TODO: make sure email and password are present in request
    dbg!("CreateAccountResponder validating");
    return Ok(200);
  }

  fn build_response(
    &self,
    request: &mut Request,
    params: &HashMap<String, String>,
    validation_code: u16,
  ) -> Result<Response, u16> {
    if validation_code != 200 {/* fall down into 400 response */}
    else {
      let mut message = String::new();
      match &mut request.message_body {
        Some(body_reader) => {
          match serde_json::from_reader::<_,CreateAccountForm>(body_reader) {
            Ok(form) => {
              match self.auth_manager.create_account(form.email, form.pass) {
                Ok(_account) => {
                  // simply return 200 here.  the next step is for the user to verify via email
                  let static_responder = StaticResponder::from_standard_code(200);
                  return static_responder.build_response(request, params, 200);
                }
                Err(error) => {
                  // convert the WebeAuth error down into something meaningful
                }
              }
            }
            Err(_error) => {/* fall down into 400 response */}
          }
        }
        None => {/* fall down into 400 response */}
      }
    }
    // TODO: Have common code-based responses be constants
    let static_responder = StaticResponder::from_standard_code(401);
    return static_responder.build_response(request, params, 401);
  }
}
