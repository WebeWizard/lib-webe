use serde::Deserialize;
use serde_json::Result;

#[derive(Deserialize)]
pub struct LoginForm {
    email: String,
    password: String,
}
