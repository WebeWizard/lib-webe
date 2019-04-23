use crate::constants::SECONDS_30_DAYS;
use crate::schema::webe_accounts;

use std::time::{SystemTime, UNIX_EPOCH};

use bcrypt::DEFAULT_COST;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable, Insertable)]
#[table_name="webe_accounts"]
pub struct Account {
  pub id: Vec<u8>, // must be unique
  pub email: String, // must be unique
  pub secret: String, // TODO: find a way to make these NOT pub
  pub secret_timeout: u32,
  pub verified: bool,
  verify_code: Option<String>, // a random verification code.  Once verified, make it None
  verify_timeout: Option<u32> // once verified, make it None.  seconds since unix epoch
  // TODO: Add a 'banned' flag
}

#[derive(Debug)]
pub enum AccountError {
  InvalidEmail, // email address does not meet validation standards
  InvalidPassword, // password does not meet validation standards
  AlreadyExists, // account with given email address already exists
  VerifyExpired, // Verifiction code has expired
  VerifyFailed, // Verification attempt failed.
  DBError, // TODO: use diesel(?) error type here
  OtherError
}

impl Account {
  pub fn new (email: String, password: String) -> Result<Account,AccountError> {
    // TODO: Verify email address looks legit, or "not_valid_email" error.
    // TODO: Verify Password, or "password_not_valid" error.
    let new_timeout: u32 = match SystemTime::now().duration_since(UNIX_EPOCH) {
      Ok(n) => n.as_secs() as u32 + SECONDS_30_DAYS,
      Err(_) => return Err(AccountError::OtherError)
    };
    // Compute Secret
    match bcrypt::hash(password, DEFAULT_COST) {
      Ok(hash) => {
        return Ok(Account{
          id: Uuid::new_v4().as_bytes().to_vec(),
          email: email.to_lowercase(), // force lowercase
          secret: hash,
          secret_timeout: new_timeout,
          verified: false,
          verify_code: Some(thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .collect()),
          verify_timeout: Some(new_timeout)
        });
      }
      _ => return Err(AccountError::InvalidPassword)
    }    
  }
}