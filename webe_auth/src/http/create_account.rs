use crate::WebeAuth;
use webe_web::request::Request;
use webe_web::responders::static_message::StaticResponder;
use webe_web::responders::Responder;
use webe_web::response::Response;

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateAccountForm {
  pub email: String,
  pub secret: String,
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
    // TODO: make sure email and secret are present in request
    dbg!("CreateAccountResponder validating");
    return Ok(200);
  }

  fn build_response(
    &self,
    request: &mut Request,
    params: &HashMap<String, String>,
    validation_code: u16,
  ) -> Result<Response, u16> {
    if validation_code != 200 {
      /* fall down into 400 response */
    } else {
      match &mut request.message_body {
        Some(body_reader) => {
          match serde_json::from_reader::<_, CreateAccountForm>(body_reader) {
            Ok(form) => {
              match self.auth_manager.create_account(form.email, form.secret) {
                Ok(_account) => {
                  // TODO: If Debug: return 200 with the account verify code.
                  // If Production: simply return 200 here.  the next step is for the user to verify via email
                  let static_responder = StaticResponder::from_standard_code(200);
                  return static_responder.build_response(request, params, 200);
                }
                Err(_error) => {
                  // TODO: If Debug, return the server's internal error in the response
                  // If Production, just show the standard error message.
                  // convert the WebeAuth error down into something meaningful
                  let static_responder = StaticResponder::from_standard_code(500);
                  return static_responder.build_response(request, params, 500);
                }
              }
            }
            Err(_error) => { /* fall down into 500 response */ }
          }
        }
        None => { /* fall down into 500 response */ }
      }
    }
    // TODO: Have common code-based responses be constants
    let static_responder = StaticResponder::from_standard_code(500);
    return static_responder.build_response(request, params, 500);
  }
}
