use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub pass: String,
}
