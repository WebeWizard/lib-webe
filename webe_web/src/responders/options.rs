use std::collections::HashMap;

use super::Request;
use super::Responder;
use super::Response;
use super::Validation;

pub struct OptionsResponder {
  origin: String,
  methods: String,
  headers: String,
}

impl OptionsResponder {
  pub fn new(origin: String, methods: String, headers: String) -> OptionsResponder {
    OptionsResponder {
      origin: origin,
      methods: methods,
      headers: headers,
    }
  }
}

impl Responder for OptionsResponder {
  fn build_response(
    &self,
    _request: &mut Request,
    _params: &HashMap<String, String>,
    _validation: Validation,
  ) -> Result<Response, u16> {
    let mut response = Response::new(204);
    response.headers.insert(
      "Access-Control-Allow-Origin".to_owned(),
      self.origin.clone(),
    );
    response.headers.insert(
      "Access-Control-Allow-Methods".to_owned(),
      self.methods.clone(),
    );
    response.headers.insert(
      "Access-Control-Allow-Headers".to_owned(),
      self.headers.clone(),
    );
    return Ok(response);
  }
}
