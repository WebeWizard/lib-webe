use crate::schema::webe_accounts;
use super::utility;

use bcrypt::DEFAULT_COST;
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable, Insertable)]
#[table_name="webe_accounts"]
pub struct Account {
  pub id: Vec<u8>, // must be unique
  pub email: String, // must be unique
  pub secret: String, // TODO: find a way to make these NOT pub
  pub secret_timeout: u32,
  pub verified: bool,
  pub verify_code: Option<String>, // a random verification code.  Once verified, make it None
  verify_timeout: Option<u32> // once verified, make it None.  seconds since unix epoch
  // TODO: Add a 'banned' flag
}

#[derive(Debug)]
pub enum AccountError {
  BadHash,
  OtherError
}

impl Account {
  pub fn new (email: String, password: String) -> Result<Account,AccountError> {
    // generate new verification code
    match utility::new_verify_code() {
      Ok(new_code) => {
        // Compute Secret
        match bcrypt::hash(password, DEFAULT_COST) {
          Ok(hash) => {
            return Ok(Account{
              id: Uuid::new_v4().as_bytes().to_vec(),
              email: email.to_lowercase(), // force lowercase
              secret: hash,
              secret_timeout: new_code.1,
              verified: false,
              verify_code: Some(new_code.0),
              verify_timeout: Some(new_code.1)
            });
          }
          _ => return Err(AccountError::BadHash)
        }    
      },
      Err(_err) => return Err(AccountError::OtherError)
    }
  }
}