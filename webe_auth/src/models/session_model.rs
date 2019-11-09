use super::account_model::Account;
use crate::constants::SECONDS_30_DAYS;
use crate::schema::webe_sessions;

use std::time::{SystemTime, UNIX_EPOCH};

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::Serialize;

#[derive(Debug, Identifiable, Queryable, Insertable, Associations, Serialize)]
#[primary_key(token)]
#[belongs_to(Account, foreign_key = "account_id")]
#[table_name = "webe_sessions"]
pub struct Session {
  pub token: String,
  pub account_id: Vec<u8>,
  timeout: u32, // based on last time credentials were provided
}

#[derive(Debug)]
pub enum SessionError {
  OtherError,
}

impl Session {
  // creates a new session
  // assumes that the user has provided valid credentials
  pub fn new(account_id: &Vec<u8>) -> Result<Session, SessionError> {
    let new_timeout: u32 = match SystemTime::now().duration_since(UNIX_EPOCH) {
      Ok(n) => n.as_secs() as u32 + SECONDS_30_DAYS,
      Err(_) => return Err(SessionError::OtherError),
    };
    return Ok(Session {
      token: thread_rng().sample_iter(&Alphanumeric).take(30).collect(),
      account_id: account_id.to_vec(),
      timeout: new_timeout,
    });
  }

  // check if the timeout has expired
  pub fn is_expired(&self) -> Result<bool, SessionError> {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
      Ok(n) => Ok((n.as_secs() as u32) > self.timeout),
      Err(_) => return Err(SessionError::OtherError),
    }
  }
}
