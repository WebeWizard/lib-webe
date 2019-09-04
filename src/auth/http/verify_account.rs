use crate::auth::WebeAuth;
use crate::http::request::Request;
use crate::http::responders::static_message::StaticResponder;
use crate::http::responders::Responder;
use crate::http::response::Response;

use std::collections::HashMap;

pub struct VerifyAccountResponder<'w> {
  auth_manager: &'w WebeAuth,
  token_name: String,
}

impl<'w> VerifyAccountResponder<'w> {
  pub fn new(auth_manager: &'w WebeAuth, token_name: String) -> VerifyAccountResponder<'w> {
    VerifyAccountResponder {
      auth_manager: auth_manager,
      token_name: token_name,
    }
  }
}

impl<'w> Responder for VerifyAccountResponder<'w> {
  fn validate(&self, _request: &Request, _params: &HashMap<String, String>) -> Result<u16, u16> {
    dbg!("VerifyAccountResponder validating");
    return Ok(200);
  }

  fn build_response(
    &self,
    request: &mut Request,
    params: &HashMap<String, String>,
    _validation_code: u16,
  ) -> Result<Response, u16> {
    match params.get(&self.token_name) {
      Some(code) => {
        match self.auth_manager.verify_account(code) {
          Ok(()) => {
            let static_responder = StaticResponder::from_standard_code(200);
            return static_responder.build_response(request, params, 200);
          }
          Err(error) => {
            // TODO: using error debug for now
            // convert the WebeAuth error down into something meaningful
            let static_responder = StaticResponder::new(500, format!("{:?}", error));
            return static_responder.build_response(request, params, 500);
          }
        }
      }
      None => {
        // bad request
        let static_responder = StaticResponder::from_standard_code(400);
        return static_responder.build_response(request, params, 400);
      }
    }
  }
}
