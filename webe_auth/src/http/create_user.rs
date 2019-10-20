use crate::WebeAuth;
use webe_web::request::Request;
use webe_web::responders::static_message::StaticResponder;
use webe_web::responders::Responder;
use webe_web::response::Response;
use webe_web::status::Status;
use webe_web::validation::Validation;
use webe_web::validation::ValidationResult;

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateUserForm {
  pub name: String,
}

pub struct CreateUserResponder<'w> {
  auth_manager: &'w WebeAuth,
}

impl<'w> CreateUserResponder<'w> {
  pub fn new(auth_manager: &'w WebeAuth) -> CreateUserResponder<'w> {
    CreateUserResponder {
      auth_manager: auth_manager,
    }
  }
}

impl<'w> Responder for CreateUserResponder<'w> {
  fn validate(&self, request: &Request, _params: &HashMap<String, String>) -> ValidationResult {
    dbg!("CreateUserResponder validating");
    match request.headers.get("x-auth-token") {
      Some(session_token) => match self.auth_manager.get_session(session_token) {
        Ok(session) => match session.is_expired() {
          Ok(valid) => {
            if valid {
              return Ok(Some(Box::new(session)));
            } else {
              return Err(Status::from_standard_code(401));
            }
          }
          Err(_error) => {}
        },
        Err(_error) => {}
      },
      None => {}
    }
    return Err(Status::from_standard_code(500));
  }

  fn build_response(
    &self,
    request: &mut Request,
    params: &HashMap<String, String>,
    validation: Validation,
  ) -> Result<Response, u16> {
    let static_responder = match &mut request.message_body {
      Some(body_reader) => {
        match serde_json::from_reader::<_, CreateUserForm>(body_reader) {
          Ok(form) => {
            // TODO: use session from validation instead of getting it again
            match request.headers.get("webe-auth") {
              Some(session_token) => {
                match self.auth_manager.get_session(session_token) {
                  Ok(session) => {
                    match self
                      .auth_manager
                      .create_user(&session.account_id, form.name)
                    {
                      // standard uuids shouldn't need this, but oh well
                      Ok(user) => match String::from_utf8(user.account_id) {
                        Ok(account_id) => StaticResponder::new(200, account_id),
                        Err(_error) => StaticResponder::from_standard_code(500),
                      },
                      Err(error) => {
                        // TODO: using error debug for now
                        // convert the WebeAuth error down into something meaningful
                        StaticResponder::new(500, format!("{:?}", error))
                      }
                    }
                  }
                  Err(error) => {
                    // TODO: using error debug for now
                    // convert the WebeAuth error down into something meaningful
                    StaticResponder::new(500, format!("{:?}", error))
                  }
                }
              }
              None => StaticResponder::from_standard_code(500),
            }
          }
          Err(_error) => StaticResponder::from_standard_code(500),
        }
      }
      None => StaticResponder::from_standard_code(500),
    };
    return static_responder.build_response(request, params, None);
  }
}
