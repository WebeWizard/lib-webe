use std::boxed::Box;
use std::collections::HashMap;
use std::io::Cursor;

use async_trait::async_trait;

use super::Request;
use super::Responder;
use super::Response;
use super::Status;
use super::Validation;

/// Responds with a fixed status code and a static message body.
pub struct StaticResponder {
    status_code: u16,
    message: String, // TODO: take a reference instead of owning it
}

impl StaticResponder {
    /// Creates a responder that always returns `status_code` with `message`.
    pub fn new(status_code: u16, message: String) -> StaticResponder {
        StaticResponder {
            status_code,
            message,
        }
    }

    /// Creates a responder from a [`Status`], using its code and reason phrase.
    pub fn from_status(status: Status) -> StaticResponder {
        StaticResponder {
            status_code: status.code,
            message: status.reason,
        }
    }

    /// Creates a responder for a standard status code, using its reason phrase
    /// as the message body.
    pub fn from_standard_code(status_code: u16) -> StaticResponder {
        let status = Status::from_standard_code(status_code);
        StaticResponder::from_status(status)
    }

    /// Builds the response directly, without needing a request, params, or
    /// validation. Useful for rendering static error pages.
    pub fn quick_response(&self) -> Response {
        let bytes = self.message.clone().into_bytes();
        let mut headers = HashMap::<String, String>::new();
        headers.insert("Content-Length".to_owned(), bytes.len().to_string());
        headers.insert("Content-Type".to_owned(), "text/html".to_owned());
        let mut response = Response::new(self.status_code);
        response.headers = headers;
        response.message_body = Some(Box::pin(Cursor::new(bytes)));
        response
    }
}

#[async_trait]
impl Responder for StaticResponder {
    async fn build_response(
        &self,
        _request: &mut Request,
        _params: &Vec<(String, String)>,
        _validation: Validation,
    ) -> Result<Response, u16> {
        Ok(self.quick_response())
    }
}
