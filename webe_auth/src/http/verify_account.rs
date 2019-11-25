use crate::AuthManager;
use crate::WebeAuth;
use serde::Deserialize;
use serde_json;
use webe_web::request::Request;
use webe_web::responders::Responder;
use webe_web::response::Response;
use webe_web::validation::Validation;

use std::collections::HashMap;
use std::io::Cursor;

#[derive(Debug, Deserialize)]
pub struct VerifyForm {
  pub email: String,
  pub pass: String,
  pub code: String,
}

pub struct VerifyAccountResponder<'w> {
  auth_manager: &'w WebeAuth<'w>,
}

impl<'w> VerifyAccountResponder<'w> {
  pub fn new(auth_manager: &'w WebeAuth) -> VerifyAccountResponder<'w> {
    VerifyAccountResponder {
      auth_manager: auth_manager,
    }
  }
}

impl<'w> Responder for VerifyAccountResponder<'w> {
  // ALWAYS RETURN Ok(200) or Err(401) TO PREVENT LEAKING API INFORMATION
  fn build_response(
    &self,
    request: &mut Request,
    _params: &HashMap<String, String>,
    _validation: Validation,
  ) -> Result<Response, u16> {
    // read and deserialize the body
    match &mut request.message_body {
      Some(body_reader) => {
        match serde_json::from_reader::<_, VerifyForm>(body_reader) {
          Ok(form) => {
            dbg!(&form);
            match self
              .auth_manager
              .verify_account(&form.email, &form.pass, &form.code)
            {
              Ok(session) => {
                match serde_json::to_string(&session) {
                  Ok(body) => {
                    let mut response = Response::new(200);
                    dbg!(&body);
                    response
                      .headers
                      .insert("Content-Type".to_owned(), "application/json".to_owned());
                    response
                      .headers
                      .insert("Content-Length".to_owned(), body.len().to_string());
                    response.message_body = Some(Box::new(Cursor::new(body)));
                    return Ok(response);
                  }
                  Err(_error) => {
                    dbg!(_error); /* fall down into 401 response */
                  }
                };
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
