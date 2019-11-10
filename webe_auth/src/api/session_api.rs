use std::time::{SystemTime, UNIX_EPOCH};

use diesel::prelude::*;
use diesel::result::Error as DieselError;

use crate::schema::webe_sessions;

use super::account_api;
use super::session_model::{Session, SessionError};

#[derive(Debug)]
pub enum SessionApiError {
  GenericLoginError,
  AccountNotVerified,
  DBError(DieselError), // errors from interacting with database
  SecretExpired,
  SessionError(SessionError),
  SessionExpired,
  OtherError,
}

pub fn find<T>(connection: &T, session_token: &String) -> Result<Session, SessionApiError>
where
  T: diesel::Connection<Backend = diesel::mysql::Mysql>, // TODO: make this backend generic
{
  match webe_sessions::table.find(session_token).first(connection) {
    Ok(session) => return Ok(session),
    Err(err) => return Err(SessionApiError::DBError(err)),
  }
}

// authenticates an account and returns a Session
// TODO: if not verified, ask if we should resend verification email
// TODO: if secret is expired.. have the user choose a new secret
pub fn login<T>(
  connection: &T,
  email_address: &String,
  secret: &String,
) -> Result<Session, SessionApiError>
where
  T: diesel::Connection<Backend = diesel::mysql::Mysql>, // TODO: make this backend generic
{
  // fetch account model matching email address
  match account_api::find_by_email(connection, email_address) {
    Ok(fetched_account) => {
      if fetched_account.verified {
        // - verify secret and secret with bcrypt
        match bcrypt::verify(secret, &fetched_account.secret) {
          Ok(verified_result) => {
            if verified_result {
              // check to see if the secret has expired
              match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(n) => {
                  if fetched_account.secret_timeout > n.as_secs() as u32 {
                    return create_session(connection, &fetched_account.id);
                  } else {
                    return Err(SessionApiError::SecretExpired);
                  }
                }
                Err(_err) => return Err(SessionApiError::OtherError),
              }
            } else {
              return Err(SessionApiError::GenericLoginError);
            }
          }
          Err(_err) => return Err(SessionApiError::GenericLoginError),
        }
      } else {
        return Err(SessionApiError::AccountNotVerified);
      }
    }
    //  - if account doesn't exist return generic login error
    Err(_err) => return Err(SessionApiError::GenericLoginError),
  }
}

// creates a new session for the account.
// note, this should only be performed as part of log-in process.
fn create_session<T>(connection: &T, acc_id: &Vec<u8>) -> Result<Session, SessionApiError>
where
  T: diesel::Connection<Backend = diesel::mysql::Mysql>, // TODO: make this backend generic
{
  match Session::new(acc_id) {
    Ok(new_session) => {
      match diesel::insert_into(webe_sessions::table) // using the ::table format here to avoid the 'use'
        .values(&new_session)
        .execute(connection)
      {
        Ok(_) => return Ok(new_session),
        Err(err) => return Err(SessionApiError::DBError(err)),
      }
    }
    Err(err) => return Err(SessionApiError::SessionError(err)),
  }
}

pub fn delete_session<T>(connection: &T, session_token: &String) -> Result<(), SessionApiError>
where
  T: diesel::Connection<Backend = diesel::mysql::Mysql>, // TODO: make this backend generic
{
  match diesel::delete(webe_sessions::table.find(session_token)).execute(connection) {
    Ok(_) => return Ok(()),
    Err(err) => return Err(SessionApiError::DBError(err)),
  }
}
