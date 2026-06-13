use crate::AuthManager;
use crate::WebeAuth;
use tokio::sync::Mutex;
use webe_web::request::Request;
use webe_web::responders::Responder;
use webe_web::response::Response;
use webe_web::validation::Validation;

use async_trait::async_trait;
use std::io::Cursor;
use std::sync::Arc;

use serde::Deserialize;
use serde_json;
use tokio::io::AsyncReadExt;

#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub pass: String,
}

pub struct LoginResponder {
    auth_manager: Arc<Mutex<WebeAuth>>,
}

impl LoginResponder {
    pub fn new(auth_manager: Arc<Mutex<WebeAuth>>) -> LoginResponder {
        LoginResponder {
            auth_manager: auth_manager,
        }
    }
}

#[async_trait]
impl Responder for LoginResponder {
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
                match serde_json::from_reader::<_, LoginForm>(body.as_slice()) {
                    Ok(form) => {
                        match self
                            .auth_manager
                            .lock()
                            .await
                            .login(&form.email, &form.pass)
                        {
                            Ok(session) => {
                                match serde_json::to_string(&session) {
                                    Ok(body) => {
                                        let mut response = Response::new(200);
                                        response.headers.insert(
                                            "Content-Type".to_owned(),
                                            "application/json".to_owned(),
                                        );
                                        response.headers.insert(
                                            "Content-Length".to_owned(),
                                            body.len().to_string(),
                                        );
                                        response.message_body = Some(Box::pin(Cursor::new(body)));
                                        return Ok(response);
                                    }
                                    Err(_error) => {
                                        dbg!(_error); /* fall down into 401 response */
                                    }
                                };
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
