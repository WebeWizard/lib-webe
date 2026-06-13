use std::sync::Arc;

use crate::AuthManager;
use crate::WebeAuth;
use tokio::sync::Mutex;
use webe_web::request::Request;
use webe_web::responders::Responder;
use webe_web::response::Response;
use webe_web::validation::Validation;

use async_trait::async_trait;

use serde::Deserialize;
use serde_json;
use tokio::io::AsyncReadExt;

#[derive(Deserialize)]
pub struct LogoutForm {
    pub token: String,
}

pub struct LogoutResponder {
    auth_manager: Arc<Mutex<WebeAuth>>,
}

impl LogoutResponder {
    pub fn new(auth_manager: Arc<Mutex<WebeAuth>>) -> LogoutResponder {
        LogoutResponder {
            auth_manager: auth_manager,
        }
    }
}

#[async_trait]
impl Responder for LogoutResponder {
    // ALWAYS RETURN Ok(200) or Err(401) TO PREVENT LEAKING API INFORMATION
    async fn build_response(
        &self,
        request: &mut Request,
        _params: &Vec<(String, String)>,
        _validation: Validation,
    ) -> Result<Response, u16> {
        match &mut request.message_body {
            Some(body_reader) => {
                let mut body = Vec::<u8>::new();
                // read the entire body or error.
                // TODO: improve workaround for serde not being able to handle async
                body_reader
                    .read_to_end(&mut body)
                    .await
                    .map_err(|_e| 400u16)?;
                match serde_json::from_reader::<_, LogoutForm>(body.as_slice()) {
                    Ok(form) => {
                        match self.auth_manager.lock().await.logout(&form.token) {
                            Ok(_) => {
                                let response = Response::new(200);
                                return Ok(response);
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
        return Err(401);
    }
}
