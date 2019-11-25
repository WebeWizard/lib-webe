use crate::constants::SECONDS_30_DAYS;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

pub fn new_verify_code() -> Result<(String, u32), SystemTimeError> {
  let new_timeout: u32 = match SystemTime::now().duration_since(UNIX_EPOCH) {
    Ok(n) => n.as_secs() as u32 + SECONDS_30_DAYS,
    Err(err) => return Err(err),
  };
  let new_code: String = thread_rng().sample_iter(&Alphanumeric).take(30).collect();
  return Ok((new_code, new_timeout));
}

// Used to serialize non-strings as strings
use serde::{de, Deserialize, Deserializer, Serializer};
use std::fmt::Display;
use std::str::FromStr;

pub fn serialize_as_string<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
  T: Display,
  S: Serializer,
{
  serializer.collect_str(value)
}

pub fn deserialize_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
  T: FromStr,
  T::Err: Display,
  D: Deserializer<'de>,
{
  String::deserialize(deserializer)?
    .parse()
    .map_err(de::Error::custom)
}
