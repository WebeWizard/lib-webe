// wraps another responder and ensures that the request contains a valid Session.
// passes the session along to the internal responder's 'validate function so that it can make
// - extra decisions based on the session ID

use std::sync::Arc;

use crate::{AuthError, AuthManager, WebeAuth};
use tokio::sync::Mutex;
use webe_web::request::Request;
use webe_web::responders::Responder;
use webe_web::response::Response;
use webe_web::status::Status;
use webe_web::validation::{Validation, ValidationResult};

use async_trait::async_trait;

pub struct SecureResponder<R: Responder> {
    auth_manager: Arc<Mutex<WebeAuth>>,
    internal_responder: R,
}

impl<R: Responder> SecureResponder<R> {
    pub fn new(auth_manager: Arc<Mutex<WebeAuth>>, internal_responder: R) -> SecureResponder<R>
    where
        R: Responder,
    {
        SecureResponder {
            auth_manager: auth_manager,
            internal_responder: internal_responder,
        }
    }
}

#[async_trait]
impl<R: Responder> Responder for SecureResponder<R> {
    async fn validate(
        &self,
        request: &Request,
        params: &Vec<(String, String)>,
        _validation: Validation,
    ) -> ValidationResult {
        // make sure session header belongs to a valid session
        match &request.headers {
            Some(headers) => {
                match headers.get("x-webe-token") {
                    Some(token) => {
                        // pass the session along to internal validation
                        match self.auth_manager.lock().await.find_valid_session(token) {
                            Ok(session) => {
                                return self
                                    .internal_responder
                                    .validate(request, params, Some(Box::new(session)))
                                    .await;
                            }
                            Err(error) => match error {
                                // TODO: match session error's timeout vs other (like the system clock error)
                                AuthError::SessionError(_error) => {
                                    return Err(Status::from_standard_code(403));
                                }
                                _ => return Err(Status::from_standard_code(500)),
                            },
                        }
                    }
                    None => return Err(Status::from_standard_code(403)),
                }
            }
            None => return Err(Status::from_standard_code(403)),
        }
    }

    async fn build_response(
        &self,
        request: &mut Request,
        params: &Vec<(String, String)>,
        validation: Validation,
    ) -> Result<Response, u16> {
        return self
            .internal_responder
            .build_response(request, params, validation)
            .await;
    }
}
