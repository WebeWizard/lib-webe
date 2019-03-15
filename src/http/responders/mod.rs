pub mod file;
pub mod static_message;

use std::collections::HashMap;

use super::status::Status;
use super::request::Request;
use super::response::Response;

pub trait Responder: Send+Sync {
    // tests if the request is worth responding to. Ok(status_code) or Err(status_code)
    fn validate(&self, request: &Request, params: &HashMap<String,String>) -> Result<u16,u16>;
    // if validate is OK, tries to respond using the supplied status code.
    // returns a Response, or a new status code to fall back to.
    fn build_response(&self, request: &Request, params: &HashMap<String,String>, validation_code: u16) -> Result<Response,u16>;
}