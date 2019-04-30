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
pub mod models;

pub mod utility;

use diesel::prelude::*;
use diesel::r2d2;
use diesel::r2d2::{ConnectionManager, Pool, PoolError};
use diesel::result::Error as DieselError;

use api::account_api;
use api::account_api::AccountApiError;
use api::user_api;
use api::user_api::UserApiError;
use api::session_api;
use api::session_api::SessionApiError;
use models::account_model::Account;
use models::session_model::Session;
use models::user_model::User;

pub struct WebeAuth {
    pub con_pool: r2d2::Pool<r2d2::ConnectionManager<MysqlConnection>>
}

#[derive(Debug)]
pub enum WebeAuthError {
    AccountApiError(AccountApiError),
    DBConnError(ConnectionError),
    DBError(DieselError),
    OtherError,
    PoolError(PoolError),
    SessionApiError(SessionApiError),
    UserApiError(UserApiError)
}

impl WebeAuth {
    pub fn new (database_url: &String) -> Result<WebeAuth,WebeAuthError> {
        let connection_manager = ConnectionManager::new(database_url.as_str());
        // build the connection pool
        match Pool::builder()
            .max_size(10)
            .build(connection_manager) {
                Ok(pool) => {
                    return Ok(WebeAuth { con_pool: pool });
                },
                Err(err) => return Err(WebeAuthError::PoolError(err))
        }
    }

    pub fn create_account (&self, user_name: String, new_email: String, password: String) -> Result<Account,WebeAuthError> {
        match self.con_pool.get() {
            Ok(connection) => {
                // TODO:  vaidate user_name, email, password, etc
                match account_api::create_account(&connection, user_name, new_email, password) {
                    Ok(new_account) => return Ok(new_account),
                    Err(err) => return Err(WebeAuthError::AccountApiError(err))
                }
            },
            Err(err) => return Err(WebeAuthError::PoolError(err))
        }
    }

    pub fn reset_verification (&self, account_id: &Vec<u8>) -> Result<(),WebeAuthError> {
        match self.con_pool.get() {
            Ok(connection) => {
                // TODO:  test to see if account is already verified
                match account_api::reset_verification(&connection, &account_id) {
                    Ok(_) => return Ok(()),
                    Err(err) => return Err(WebeAuthError::AccountApiError(err))
                }
            },
            Err(err) => return Err(WebeAuthError::PoolError(err))
        }
    }

    pub fn delete_account (&self, account_id: &Vec<u8>) -> Result<(),WebeAuthError> {
        match self.con_pool.get() {
            Ok(connection) => {
                // TODO:  Make sure the requesting Session has permission to delete this account
                match account_api::delete_account(&connection, account_id) {
                    Ok(_) => return Ok(()),
                    Err(err) => return Err(WebeAuthError::AccountApiError(err))
                }
            },
            Err(err) => return Err(WebeAuthError::PoolError(err))
        }
    }

    pub fn get_account_by_email (&self, email_address: &String) -> Result<Account,WebeAuthError> {
        match self.con_pool.get() {
            Ok(connection) => {
                match account_api::find_by_email(&connection, email_address) {
                    Ok(account) => return Ok(account),
                    Err(err) => return Err(WebeAuthError::AccountApiError(err))
                }
            },
            Err(err) => return Err(WebeAuthError::PoolError(err))
        }
    }

    pub fn create_user (&self, acc_id: &Vec<u8>, name: String) -> Result<User,WebeAuthError> {
        match self.con_pool.get() {
            Ok(connection) => {
                // TODO:  vaidate user_name, email, password, etc
                match user_api::create_user(&connection, acc_id, name) {
                    Ok(new_user) => return Ok(new_user),
                    Err(err) => return Err(WebeAuthError::UserApiError(err))
                }
            },
            Err(err) => return Err(WebeAuthError::PoolError(err))
        }
    }

    pub fn get_user_by_name (&self, acc_id: &Vec<u8>, name: &String) ->  Result<User,WebeAuthError> {
        match self.con_pool.get() {
            Ok(connection) => {
                match user_api::find_by_name(&connection, acc_id, name) {
                    Ok(user) => return Ok(user),
                    Err(err) => return Err(WebeAuthError::UserApiError(err))
                }
            },
            Err(err) => return Err(WebeAuthError::PoolError(err))
        }
    }

    pub fn delete_user (&self, user_id: &Vec<u8>) -> Result<(),WebeAuthError> {
        match self.con_pool.get() {
            Ok(connection) => {
                // TODO:  Make sure the requesting Session has permission to delete this account
                match user_api::delete_user(&connection, user_id) {
                    Ok(_) => return Ok(()),
                    Err(err) => return Err(WebeAuthError::UserApiError(err))
                }
            },
            Err(err) => return Err(WebeAuthError::PoolError(err))
        }
    }
        
    pub fn login (&self, email_address: String, pass: String) -> Result<Session,WebeAuthError> {
        match self.con_pool.get() {
            Ok(connection) => {
                match session_api::login(&connection, &email_address, &pass) {
                    Ok(new_session) => return Ok(new_session),
                    Err(err) => return Err(WebeAuthError::SessionApiError(err))
                }
            },
            Err(err) => return Err(WebeAuthError::PoolError(err))
        }
    }
}