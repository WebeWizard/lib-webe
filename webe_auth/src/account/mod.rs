use crate::schema::webe_accounts;
use crate::utility;
use crate::AuthError;

use bcrypt::DEFAULT_COST;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Identifiable, Insertable, Queryable)]
#[table_name = "webe_accounts"]
pub struct Account {
  pub id: u64,        // must be unique
  pub email: String,  // must be unique
  pub secret: String, // TODO: find a way to make these NOT pub
  pub secret_timeout: u32,
  pub verify_code: Option<String>, // a random verification code.  Once verified, make it None
  verify_timeout: Option<u32>,     // once verified, make it None.  seconds since unix epoch
                                   // TODO: Add a 'banned' flag/reason string
}

#[derive(Debug)]
pub enum AccountError {
  BadPass,
  BadHash,
  NotVerified,
  VerifyError(VerifyError),
  OtherError,
}

#[derive(Debug)]
pub enum VerifyError {
  BadCode,
  AlreadyVerified,
  TimeoutExpired,
}

impl From<AccountError> for AuthError {
  fn from(err: AccountError) -> AuthError {
    AuthError::AccountError(err)
  }
}

impl From<VerifyError> for AccountError {
  fn from(err: VerifyError) -> AccountError {
    AccountError::VerifyError(err)
  }
}

impl Account {
  pub fn new(id: u64, email: String, password: String) -> Result<Account, AccountError> {
    // generate new verification code
    match utility::new_verify_code() {
      Ok(new_code) => {
        // Compute Secret
        match bcrypt::hash(password, DEFAULT_COST) {
          Ok(hash) => {
            return Ok(Account {
              id: id,
              email: email.to_lowercase(), // force lowercase
              secret: hash,
              secret_timeout: new_code.1,
              verify_code: Some(new_code.0),
              verify_timeout: Some(new_code.1),
            });
          }
          _ => return Err(AccountError::BadHash),
        }
      }
      Err(_err) => return Err(AccountError::OtherError),
    }
  }

  pub fn check_pass(&self, test_pass: &String) -> Result<(), AccountError> {
    // hash the test_pass and see if it matches the account's secret
    match bcrypt::verify(test_pass.as_str(), self.secret.as_str()) {
      Ok(result) => {
        if result {
          return Ok(());
        } else {
          return Err(AccountError::BadPass);
        }
      }
      Err(_) => return Err(AccountError::BadHash),
    }
  }

  // check a code for an unverified account, or check to see if an account is already verified.
  pub fn check_verify(&self, test_code: Option<&String>) -> Result<(), AccountError> {
    match test_code {
      Some(test_code) => {
        // check if account is already verified
        match &self.verify_code {
          Some(existing_code) => {
            // check matching code
            if existing_code != test_code {
              return Err(AccountError::VerifyError(VerifyError::BadCode));
            }
            // check timeout not expired
            match self.verify_timeout {
              Some(existing_timeout) => match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(duration) => {
                  if duration.as_secs() as u32 > existing_timeout {
                    return Err(AccountError::VerifyError(VerifyError::TimeoutExpired));
                  } else {
                    return Ok(());
                  }
                }
                Err(_err) => return Err(AccountError::OtherError),
              },
              None => return Err(AccountError::VerifyError(VerifyError::AlreadyVerified)),
            }
          }
          None => return Err(AccountError::VerifyError(VerifyError::AlreadyVerified)),
        }
      }
      None => {
        // check to see if account is already verified.
        match &self.verify_code {
          Some(_code) => return Err(AccountError::NotVerified),
          None => return Ok(()),
        }
      }
    }
  }
}
