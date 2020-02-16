use std::collections::HashMap;

use super::Request;
use super::Responder;
use super::Response;
use super::Validation;
use super::ValidationResult;

use super::file::{FileResponder, FileResponderError};

// Used to capture stray endpoints that would normally be handled within the SPA
// Ex.  User refreshes and loads a /flash/23434455 url
pub struct SPAResponder {
  app_file_path: String,
  file_responder: FileResponder,
}

impl SPAResponder {
  // creates an inner file responder where the mount point becomes the complete app file path
  pub fn new(
    mount_point: String,
    app_file_path: String,
  ) -> Result<SPAResponder, FileResponderError> {
    match FileResponder::new(mount_point, String::new()) {
      Ok(file_responder) => Ok(SPAResponder {
        app_file_path: app_file_path,
        file_responder: file_responder,
      }),
      Err(error) => return Err(error),
    }
  }
}

impl Responder for SPAResponder {
  fn validate(
    &self,
    request: &Request,
    _params: &HashMap<String, String>,
    validation: Validation,
  ) -> ValidationResult {
    // pass on to the internal file responder, but fudge the param
    let mut fudged_params = HashMap::<String, String>::new();
    // param gets set to an empty string so that app_file_path becomes the complete path
    fudged_params.insert(String::new(), self.app_file_path.clone());
    return self
      .file_responder
      .validate(request, &fudged_params, validation);
  }

  fn build_response(
    &self,
    request: &mut Request,
    params: &HashMap<String, String>,
    validation: Validation,
  ) -> Result<Response, u16> {
    return self
      .file_responder
      .build_response(request, params, validation);
  }
}
