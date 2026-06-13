pub mod file;
pub mod options;
pub mod spa;
pub mod static_message;

use async_trait::async_trait;

use super::request::Request;
use super::response::Response;
use super::status::Status;
use super::validation::Validation;
use super::validation::ValidationResult;

#[async_trait]
pub trait Responder: Send + Sync {
    // tests if the request is worth responding to. Ok(status_code) or Err(status_code)
    // accepts a Validation to support things like a wrapped "Secure/LoggedIn responder"
    async fn validate(
        &self,
        _request: &Request,
        _params: &Vec<(String, String)>,
        validation: Validation,
    ) -> ValidationResult {
        Ok(validation) // default is to forward the validation along
    }

    // NOTE: Request is mutable!  Mostly this is so the message bufreader can be read from.
    // validation_code is used to hint to the responder what kind of response should be given
    // returns a Response, or a new status code to fall back to.
    async fn build_response(
        &self,
        request: &mut Request,
        params: &Vec<(String, String)>,
        validation: Validation,
    ) -> Result<Response, u16>;
}
