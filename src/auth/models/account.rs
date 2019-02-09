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
  email: String, // must be unique
  secret: String,
  secret_timeout: u32,
  verified: bool,
  verify_code: Option<String>, // a random verification code.  Once verified, make it None
  verify_timeout: Option<u32> // once verified, make it None.  seconds since unix epoch
}

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
          email: email,
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

  // Delete an account completely
  // Consume the session
  // pub fn delete (session: Session) -> Result<(),AccountError> {
  //   // Assume the session is valid (logic should be behind some protected guard)
  //   // delete the account from db (should cascade to all user related data)
  //   // if successful, return OK, otherwise return error
  // }

  // pub fn verify (session: &Session, verify_code: String) -> Result<(),AccountError>{
  //   // Assume the session is valid (logic should be behind some protected guard)
  //   // fetch account record from db
  //   // check current system time against verify expiration time
  //   // -- if expired, return "verify_expired" error
  //   // otherwise...
  //   // check verify_code against the one stored on record
  //   // if matches
  //   // -- set record verify_code to None
  //   // -- set record expiration to 
  //   // if not
  //   // -- return verification failed error
  // }
}