use std::sync::Arc;
use tokio::sync::Mutex;

use crate::AuthManager;
use crate::WebeAuth;
use webe_web::request::Request;
use webe_web::responders::Responder;
use webe_web::responders::static_message::StaticResponder;
use webe_web::response::Response;
use webe_web::validation::Validation;

use async_trait::async_trait;
use serde::Deserialize;

use tokio::io::AsyncReadExt;

#[derive(Deserialize)]
pub struct CreateAccountForm {
    pub email: String,
    pub secret: String,
}

pub struct CreateAccountResponder {
    auth_manager: Arc<Mutex<WebeAuth>>,
}

impl CreateAccountResponder {
    pub fn new(auth_manager: Arc<Mutex<WebeAuth>>) -> CreateAccountResponder {
        CreateAccountResponder {
            auth_manager: auth_manager,
        }
    }
}

#[async_trait]
impl Responder for CreateAccountResponder {
    async fn build_response(
        &self,
        request: &mut Request,
        params: &Vec<(String, String)>,
        _validation: Validation,
    ) -> Result<Response, u16> {
        match &mut request.message_body {
            Some(body_reader) => {
                let mut body = Vec::<u8>::new();
                body_reader
                    .read_to_end(&mut body)
                    .await
                    .map_err(|_e| 400u16)?;
                // TODO: use from_utf8 instead of lossy, that way we properly catch malformed text
                let body: String = String::from_utf8_lossy(&body).into_owned();
                match serde_json::from_reader::<_, CreateAccountForm>(body.as_bytes()) {
                    Ok(form) => {
                        dbg!("email: {}, secret: {}", &form.email, &form.secret);
                        match self
                            .auth_manager
                            .lock()
                            .await
                            .create_account(form.email, form.secret)
                        {
                            Ok(_account) => {
                                // TODO: If Debug: return 200 with the account verify code.
                                // If Production: simply return 200 here.  the next step is for the user to verify via email
                                let static_responder = StaticResponder::from_standard_code(200);
                                return static_responder
                                    .build_response(request, params, None)
                                    .await;
                            }
                            Err(_error) => {
                                // TODO: If Debug, return the server's internal error in the response
                                // If Production, just show the standard error message.
                                // convert the WebeAuth error down into something meaningful
                                let static_responder = StaticResponder::from_standard_code(500);
                                return static_responder
                                    .build_response(request, params, None)
                                    .await;
                            }
                        }
                    }
                    Err(error) => {
                        dbg!(error); /* fall down into 500 response */
                    }
                }
            }
            None => { /* fall down into 500 response */ }
        }
        // TODO: Have common code-based responses be constants
        return Err(500);
    }
}
