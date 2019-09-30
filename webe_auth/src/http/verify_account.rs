use crate::WebeAuth;
use webe_web::request::Request;
use webe_web::responders::static_message::StaticResponder;
use webe_web::responders::Responder;
use webe_web::response::Response;
use webe_web::validation::Validation;

use std::collections::HashMap;

pub struct VerifyAccountResponder<'w> {
  auth_manager: &'w WebeAuth,
  token_name: String,
}

impl<'w> VerifyAccountResponder<'w> {
  pub fn new(auth_manager: &'w WebeAuth, token_name: &str) -> VerifyAccountResponder<'w> {
    VerifyAccountResponder {
      auth_manager: auth_manager,
      token_name: token_name.to_string(),
    }
  }
}

impl<'w> Responder for VerifyAccountResponder<'w> {
  fn build_response(
    &self,
    request: &mut Request,
    params: &HashMap<String, String>,
    _validation: Validation,
  ) -> Result<Response, u16> {
    match params.get(&self.token_name) {
      Some(code) => {
        match self.auth_manager.verify_account(code) {
          Ok(()) => {
            let static_responder = StaticResponder::from_standard_code(200);
            return static_responder.build_response(request, params, None);
          }
          Err(error) => {
            // TODO: using error debug for now
            // convert the WebeAuth error down into something meaningful
            let static_responder = StaticResponder::new(500, format!("{:?}", error));
            return static_responder.build_response(request, params, None);
          }
        }
      }
      None => {
        // bad request
        let static_responder = StaticResponder::from_standard_code(400);
        return static_responder.build_response(request, params, None);
      }
    }
  }
}
