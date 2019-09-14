#[macro_use]
extern crate diesel;
extern crate lettre;
extern crate lettre_email;
extern crate rand;
extern crate r2d2;
extern crate serde;
extern crate uuid;
extern crate webe_web;

/*
  Accounts - used to authenticate a single user
  Users - contains user specific information like name, email, etc.
  Session - A result of logging in and selecting a user.
   - Contains a token for accessing user specific data
   - Contains an expiration date for timing out and forcing re-auth

  TODO - for every db table, have two users, neither of which can edit table structure/permission
   - one that can insert/update/select/delete
   - one that can only read
*/

pub mod api;
pub mod constants;
pub mod http;
pub mod models;
pub mod schema;
pub mod utility;

use diesel::prelude::*;
use diesel::r2d2 as diesel_r2d2;
use diesel::result::Error as DieselError;
use lettre::{SmtpClient, SmtpConnectionManager};
use lettre::smtp::authentication::IntoCredentials;
use lettre::smtp::error::Error as LettreError;

use api::account_api;
use api::account_api::AccountApiError;
use api::session_api;
use api::session_api::SessionApiError;
use api::user_api;
use api::user_api::UserApiError;
use models::account_model::Account;
use models::session_model::Session;
use models::user_model::User;

pub struct WebeAuth {
  pub db_conn_pool: diesel_r2d2::Pool<diesel_r2d2::ConnectionManager<MysqlConnection>>,
  pub email_conn_pool: r2d2::Pool<SmtpConnectionManager>
}

#[derive(Debug)]
pub enum WebeAuthError {
  AccountApiError(AccountApiError),
  BadUser,
  DBConnError(ConnectionError),
  DBError(DieselError),
  DBPoolError(diesel_r2d2::PoolError),
  EmailConnError(LettreError),
  OtherError,
  SessionApiError(SessionApiError),
  SessionExpired,
  UserApiError(UserApiError),
}

// TODO: create a new AuthManager trait to separate struct methods from impl

impl WebeAuth {
  pub fn new(database_url: &String) -> Result<WebeAuth, WebeAuthError> {
    let connection_manager = diesel_r2d2::ConnectionManager::new(database_url.as_str());
    // build the database connection pool
    match diesel_r2d2::Pool::builder().max_size(10).build(connection_manager) {
      Ok(db_pool) => {
        // build the email connection pool
        let email_client = SmtpClient::new_simple("smtp.gmail.com").or
          .credentials(("jleis@webewizard.com", "JLeis@2406").into_credentials());
        let email_manager = SmtpConnectionManager::new(email_client)
          .expect("Failed to create smtp email manager");
        let email_pool = r2d2::Pool::builder().max_size(10).build(email_manager);

        return Ok(
          WebeAuth {
            db_conn_pool: db_pool,
            email_conn_pool: email_pool
          }
        );
      }
      Err(err) => return Err(WebeAuthError::DBPoolError(err)),
    }
  }

  pub fn create_account(
    &self,
    new_email: String,
    password: String,
  ) -> Result<Account, WebeAuthError> {
    match self.db_conn_pool.get() {
      Ok(connection) => {
        // TODO:  validate user_name, email, password, etc
        match account_api::create_account(&connection, new_email, password) {
          Ok(new_account) => return Ok(new_account),
          Err(err) => return Err(WebeAuthError::AccountApiError(err)),
        }
      }
      Err(err) => return Err(WebeAuthError::DBPoolError(err)),
    }
  }

  pub fn verify_account(&self, code: &String) -> Result<(), WebeAuthError> {
    match self.db_conn_pool.get() {
      Ok(connection) => {
        // TODO:  test to see if account is already verified
        match account_api::verify(&connection, code) {
          Ok(_) => return Ok(()),
          Err(err) => return Err(WebeAuthError::AccountApiError(err)),
        }
      }
      Err(err) => return Err(WebeAuthError::DBPoolError(err)),
    }
  }

  pub fn reset_verification(&self, account_id: &Vec<u8>) -> Result<(), WebeAuthError> {
    match self.db_conn_pool.get() {
      Ok(connection) => {
        // TODO:  test to see if account is already verified
        match account_api::reset_verification(&connection, &account_id) {
          Ok(_) => return Ok(()),
          Err(err) => return Err(WebeAuthError::AccountApiError(err)),
        }
      }
      Err(err) => return Err(WebeAuthError::DBPoolError(err)),
    }
  }

  pub fn delete_account(&self, account_id: &Vec<u8>) -> Result<(), WebeAuthError> {
    match self.db_conn_pool.get() {
      Ok(connection) => {
        // TODO:  Make sure the requesting Session has permission to delete this account
        match account_api::delete_account(&connection, account_id) {
          Ok(_) => return Ok(()),
          Err(err) => return Err(WebeAuthError::AccountApiError(err)),
        }
      }
      Err(err) => return Err(WebeAuthError::DBPoolError(err)),
    }
  }

  pub fn get_account_by_email(&self, email_address: &String) -> Result<Account, WebeAuthError> {
    match self.db_conn_pool.get() {
      Ok(connection) => match account_api::find_by_email(&connection, email_address) {
        Ok(account) => return Ok(account),
        Err(err) => return Err(WebeAuthError::AccountApiError(err)),
      },
      Err(err) => return Err(WebeAuthError::DBPoolError(err)),
    }
  }

  pub fn create_user(&self, acc_id: &Vec<u8>, name: String) -> Result<User, WebeAuthError> {
    match self.db_conn_pool.get() {
      Ok(connection) => {
        // TODO:  vaidate user_name, email, password, etc
        match user_api::create_user(&connection, acc_id, name) {
          Ok(new_user) => return Ok(new_user),
          Err(err) => return Err(WebeAuthError::UserApiError(err)),
        }
      }
      Err(err) => return Err(WebeAuthError::DBPoolError(err)),
    }
  }

  pub fn get_user_by_name(&self, acc_id: &Vec<u8>, name: &String) -> Result<User, WebeAuthError> {
    match self.db_conn_pool.get() {
      Ok(connection) => match user_api::find_by_name(&connection, acc_id, name) {
        Ok(user) => return Ok(user),
        Err(err) => return Err(WebeAuthError::UserApiError(err)),
      },
      Err(err) => return Err(WebeAuthError::DBPoolError(err)),
    }
  }

  pub fn delete_user(&self, user_id: &Vec<u8>) -> Result<(), WebeAuthError> {
    match self.db_conn_pool.get() {
      Ok(connection) => {
        // TODO:  Make sure the requesting Session has permission to delete this account
        match user_api::delete_user(&connection, user_id) {
          Ok(_) => return Ok(()),
          Err(err) => return Err(WebeAuthError::UserApiError(err)),
        }
      }
      Err(err) => return Err(WebeAuthError::DBPoolError(err)),
    }
  }

  pub fn login(&self, email_address: &String, pass: &String) -> Result<Session, WebeAuthError> {
    match self.db_conn_pool.get() {
      Ok(connection) => match session_api::login(&connection, &email_address, &pass) {
        Ok(new_session) => return Ok(new_session),
        Err(err) => return Err(WebeAuthError::SessionApiError(err)),
      },
      Err(err) => return Err(WebeAuthError::DBPoolError(err)),
    }
  }

  pub fn get_session(&self, session_token: &String) -> Result<Session, WebeAuthError> {
    match self.db_conn_pool.get() {
      Ok(connection) => match session_api::find(&connection, &session_token) {
        Ok(new_session) => return Ok(new_session),
        Err(err) => return Err(WebeAuthError::SessionApiError(err)),
      },
      Err(err) => return Err(WebeAuthError::DBPoolError(err)),
    }
  }

  pub fn change_user(
    &self,
    session_token: &String,
    new_user_id: &Vec<u8>,
  ) -> Result<(), WebeAuthError> {
    match self.db_conn_pool.get() {
      Ok(connection) => match session_api::change_user(&connection, session_token, new_user_id) {
        Ok(new_session) => return Ok(new_session),
        Err(err) => return Err(WebeAuthError::SessionApiError(err)),
      },
      Err(err) => return Err(WebeAuthError::DBPoolError(err)),
    }
  }

  pub fn delete_session(&self, session_token: &String) -> Result<(), WebeAuthError> {
    match self.db_conn_pool.get() {
      Ok(connection) => {
        // TODO:  Make sure the requesting Session has permission to delete this account
        match session_api::delete_session(&connection, session_token) {
          Ok(_) => return Ok(()),
          Err(err) => return Err(WebeAuthError::SessionApiError(err)),
        }
      }
      Err(err) => return Err(WebeAuthError::DBPoolError(err)),
    }
  }
}
