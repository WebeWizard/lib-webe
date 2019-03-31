use crate::serde::Serialize;
use super::{Request,Response,Responder};

#[derive(Clone)]
pub struct JSONResponder<T: Serialize> {
    object: T // the object to serialize and send as the response body
}

impl Responder for JSONResponder<Serialize> {
    fn validate(&self, request: &Request, params: &HashMap<String,String>) -> Result<u16,u16> {
        // the object is already guaranteed to be serializable, so no worries here
        Ok(200)
    }

    fn build_response(&self, request: &Request, params: &HashMap<String,String>, validation_code: u16) -> Result<Response,u16> {
        serde_json::to_string(self.object).unwrap
        return Ok(response);
    }
}