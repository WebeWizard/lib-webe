use crate::constants::SECONDS_30_DAYS;

use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn new_verify_code () -> Result<(String, u32),SystemTimeError> {
  let new_timeout: u32 = match SystemTime::now().duration_since(UNIX_EPOCH) {
      Ok(n) => n.as_secs() as u32 + SECONDS_30_DAYS,
      Err(err) => return Err(err)
  };
  let new_code: String = thread_rng()
    .sample_iter(&Alphanumeric)
    .take(30)
    .collect();
  return Ok((new_code, new_timeout));
}