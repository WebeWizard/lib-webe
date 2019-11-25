use crate::AuthManager;
use crate::WebeAuth;
use webe_web::request::Request;
use webe_web::responders::static_message::StaticResponder;
use webe_web::responders::Responder;
use webe_web::response::Response;
use webe_web::validation::Validation;

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateAccountForm {
  pub email: String,
  pub secret: String,
}

pub struct CreateAccountResponder<'w> {
  auth_manager: &'w WebeAuth<'w>,
}

impl<'w> CreateAccountResponder<'w> {
  pub fn new(auth_manager: &'w WebeAuth) -> CreateAccountResponder<'w> {
    CreateAccountResponder {
      auth_manager: auth_manager,
    }
  }
}

impl<'w> Responder for CreateAccountResponder<'w> {
  fn build_response(
    &self,
    request: &mut Request,
    params: &HashMap<String, String>,
    _validation: Validation,
  ) -> Result<Response, u16> {
    match &mut request.message_body {
      Some(body_reader) => {
        match serde_json::from_reader::<_, CreateAccountForm>(body_reader) {
          Ok(form) => {
            dbg!("email: {}, secret: {}", &form.email, &form.secret);
            match self.auth_manager.create_account(form.email, form.secret) {
              Ok(_account) => {
                // TODO: If Debug: return 200 with the account verify code.
                // If Production: simply return 200 here.  the next step is for the user to verify via email
                let static_responder = StaticResponder::from_standard_code(200);
                return static_responder.build_response(request, params, None);
              }
              Err(_error) => {
                // TODO: If Debug, return the server's internal error in the response
                // If Production, just show the standard error message.
                // convert the WebeAuth error down into something meaningful
                let static_responder = StaticResponder::from_standard_code(500);
                return static_responder.build_response(request, params, None);
              }
            }
          }
          Err(error) => {
            dbg!(error); /* fall down into 500 response */
          }
        }
      }
      None => { /* fall down into 500 response */ }
    }
    // TODO: Have common code-based responses be constants
    return Err(500);
  }
}
