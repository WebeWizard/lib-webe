use super::session_model::Session;

pub enum SessionApiError {
    GenericLoginError
}

// authenticates an account and returns a Session (without user)
fn login (email: &str, secret: &str) -> Result<Session,SessionApiError> {
    // fetch account model matching email address
    //  - if doesn't exist return generic login error
    // check if verified
    //  - if not verified, ask if we should resend verification email
    // if verified, check secret
    //  - if not match, return generic login error
    // if secret is valid, check to see if it has expired
    //  - if expired, TODO:  have the user enter new password
    // if secret has not expired, create an return a new session!
    return Err(SessionApiError::GenericLoginError);
}