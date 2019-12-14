// wraps another responder and ensures that the request contains a valid Session.
// passes the session along to the internal responder's 'validate function so that it can make
// - extra decisions based on the session ID

use std::collections::HashMap;

use crate::{AuthError, AuthManager, WebeAuth};
use webe_web::request::Request;
use webe_web::responders::Responder;
use webe_web::response::Response;
use webe_web::status::Status;
use webe_web::validation::{Validation, ValidationResult};

pub struct SecureResponder<'w, R: Responder> {
  auth_manager: &'w WebeAuth<'w>,
  internal_responder: R,
}

impl<'w, R: Responder> SecureResponder<'w, R> {
  pub fn new(auth_manager: &'w WebeAuth, internal_responder: R) -> SecureResponder<'w, R>
  where
    R: Responder,
  {
    SecureResponder {
      auth_manager: auth_manager,
      internal_responder: internal_responder,
    }
  }
}

impl<'w, R: Responder> Responder for SecureResponder<'w, R> {
  fn validate(
    &self,
    request: &Request,
    params: &HashMap<String, String>,
    _validation: Validation,
  ) -> ValidationResult {
    // make sure session header belongs to a valid session
    match request.headers.get("x-webe-token") {
      Some(token) => {
        // pass the session along to internal validation
        match self.auth_manager.find_valid_session(token) {
          Ok(session) => {
            return self
              .internal_responder
              .validate(request, params, Some(Box::new(session)))
          }
          Err(error) => match error {
            // TODO: match session error's timeout vs other (like the system clock error)
            AuthError::SessionError(s_error) => return Err(Status::from_standard_code(403)),
            _ => return Err(Status::from_standard_code(500)),
          },
        }
      }
      None => return Err(Status::from_standard_code(403)),
    }
  }

  fn build_response(
    &self,
    request: &mut Request,
    params: &HashMap<String, String>,
    validation: Validation,
  ) -> Result<Response, u16> {
    self
      .internal_responder
      .build_response(request, params, validation)
  }
}
