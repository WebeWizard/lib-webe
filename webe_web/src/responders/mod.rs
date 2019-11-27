pub mod file;
pub mod options;
pub mod static_message;

use std::collections::HashMap;

use super::request::Request;
use super::response::Response;
use super::status::Status;
use super::validation::Validation;
use super::validation::ValidationResult;

pub trait Responder: Send + Sync {
  // tests if the request is worth responding to. Ok(status_code) or Err(status_code)
  fn validate(&self, _request: &Request, _params: &HashMap<String, String>) -> ValidationResult {
    Ok(None)
  }

  // NOTE: Request is mutable!  Mostly this is so the message bufreader can be read from.
  // validation_code is used to hint to the responder what kind of response should be given
  // returns a Response, or a new status code to fall back to.
  fn build_response(
    &self,
    request: &mut Request,
    params: &HashMap<String, String>,
    validation: Validation,
  ) -> Result<Response, u16>;
}
