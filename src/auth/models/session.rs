use crate::constants::SECONDS_30_DAYS;
use crate::schema::webe_sessions;
use super::user::User;

use std::time::{SystemTime, UNIX_EPOCH};

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

#[derive(Debug, Identifiable, Queryable, Insertable, Associations)]
#[primary_key(token)]
#[belongs_to(User, foreign_key="user_id")]
#[table_name="webe_sessions"]
pub struct Session {
  token: String,
  user_id: Vec<u8>,
  timeout: u32
}

pub enum SessionError {
  OtherError
}

impl Session {
  // creates a new session
  // assumes that the user has provided valid credentials
  pub fn new (user_id: &Vec<u8>) -> Result<Session,SessionError> {
    let new_timeout: u32 = match SystemTime::now().duration_since(UNIX_EPOCH) {
      Ok(n) => n.as_secs() as u32 + SECONDS_30_DAYS,
      Err(_) => return Err(SessionError::OtherError)
    };
    return Ok(Session{
      token: thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .collect(),
      user_id: user_id.to_vec(),
      timeout: new_timeout
    });
  }
}