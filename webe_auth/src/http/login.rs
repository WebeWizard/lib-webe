use crate::WebeAuth;
use webe_web::request::Request;
use webe_web::responders::static_message::StaticResponder;
use webe_web::responders::Responder;
use webe_web::response::Response;
use webe_web::validation::Validation;

use std::collections::HashMap;
use std::io::Cursor;

use serde::Deserialize;
use serde_json;

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
  // ALWAYS RETURN Ok(200) or Err(401) TO PREVENT LEAKING API INFORMATION
  fn build_response(
    &self,
    request: &mut Request,
    params: &HashMap<String, String>,
    _validation: Validation,
  ) -> Result<Response, u16> {
    match &mut request.message_body {
      Some(body_reader) => {
        match serde_json::from_reader::<_, LoginForm>(body_reader) {
          Ok(form) => {
            match self.auth_manager.login(&form.email, &form.pass) {
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
    let static_responder = StaticResponder::from_standard_code(401);
    return static_responder.build_response(request, params, None);
  }
}
