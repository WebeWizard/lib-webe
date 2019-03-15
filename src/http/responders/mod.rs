pub mod file;
pub mod static_message;

use std::collections::HashMap;

use super::status::Status;
use super::request::Request;
use super::response::Response;

pub trait Responder: Send+Sync {
    fn validate(&self, request: &Request, params: &HashMap<String,String>) -> bool;
    fn build_response(&self, request: &Request, params: &HashMap<String,String>) -> Response; // TODO: should this be a result?
}