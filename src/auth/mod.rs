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

pub mod models;

use std::time::{SystemTime, UNIX_EPOCH};

use diesel::prelude::*;
use diesel::r2d2;
use diesel::r2d2::{ConnectionManager, Pool, PoolError};
use diesel::result::{DatabaseErrorKind, Error as DieselError};

use super::schema::{webe_accounts, webe_users, webe_sessions};
use super::schema::webe_accounts::dsl::*;
use models::account::{Account, AccountError};
use models::session::{Session, SessionError};
use models::user::{User, UserError};

pub struct WebeAuth {
    pub con_pool: r2d2::Pool<r2d2::ConnectionManager<MysqlConnection>>
}

#[derive(Debug)]
pub enum WebeAuthError {
    Account(AccountError),
    DBConnError(ConnectionError),
    DBError(DieselError),
    LoginError, // generic login error
    OtherError,
    PasswordExpired,
    PoolError(PoolError),
    Session(SessionError),
    SessionNotFound,
    SessionExpired,
    NotVerifiedError,
    User(UserError)
}

impl WebeAuth {
    pub fn new(database_url: &String) -> Result<WebeAuth,WebeAuthError> {
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

    pub fn create_account(&self, user_name: String, new_email: String, password: String) -> Result<Account,WebeAuthError> {
        // create the new account
        match Account::new(new_email, password) {
            Ok(new_account) => {
                // attempt to insert new account into database
                match self.con_pool.get() {
                    Ok(connection) => {
                        match diesel::insert_into(webe_accounts::table)
                            .values(&new_account)
                            .execute(&connection) {
                                Ok(_) => {
                                    match User::new(&new_account.id, user_name) {
                                        Ok(new_user) => {
                                            // attempt to insert default user into database
                                            match diesel::insert_into(webe_users::table)
                                                .values(&new_user)
                                                .execute(&connection) {
                                                    Ok(_) => return Ok(new_account),
                                                    Err(err) => return Err(WebeAuthError::DBError(err))
                                            }
                                        },
                                        Err(err) => return Err(WebeAuthError::User(err))
                                    }
                                },
                                Err(err) => {
                                    // check for unique constraint (currently only on PK and email)
                                    match err {
                                        DieselError::DatabaseError(dberr, _) => {
                                            match dberr {
                                                DatabaseErrorKind::UniqueViolation =>
                                                    return Err(WebeAuthError::Account(AccountError::AlreadyExists)),
                                                _ => return Err(WebeAuthError::DBError(err))
                                            }
                                        },
                                        _ => return Err(WebeAuthError::DBError(err))
                                    }
                                }
                        }
                    },
                    Err(err) => return Err(WebeAuthError::PoolError(err))
                }
            },
            Err(error) => return Err(WebeAuthError::Account(error))
        }
    }
    
    // attempts to update the db session record with a new user and if successful, mutates the session
    pub fn change_user(&self, current_session: &mut Session, user_id: &Vec<u8>) -> Result<(),WebeAuthError> {
        match self.con_pool.get() {
            Ok(connection) => {
                match diesel::update(current_session as &Session) // diesel doesn't like mutable references
                    .set(webe_sessions::user_id.eq(user_id))
                    .execute(&connection) {
                        Ok(1) => { // only 1 record updated is a success
                            current_session.user_id = Some(user_id.to_vec());
                            return Ok(());
                        },
                        Ok(_) => return Err(WebeAuthError::SessionNotFound),
                        Err(err) => return Err(WebeAuthError::DBError(err))
                }
            },
            Err(err) => return Err(WebeAuthError::PoolError(err))
        }
    }
        
    pub fn login(&self, login_email: String, pass: String, user_id: &Vec<u8>) -> Result<Session,WebeAuthError> {
        // fetch account from db using email
        match self.con_pool.get() {
            Ok(connection) => {
                match webe_accounts.filter(webe_accounts::email.eq(login_email))
                    .first::<Account>(&connection) {
                        Ok(fetched_account) => {
                            // - verify password and secret with bcrypt
                            match bcrypt::verify(pass, &fetched_account.secret) {
                                Ok(verified_result) => {
                                    if verified_result { // password matches
                                        // see if password is expired
                                        match SystemTime::now().duration_since(UNIX_EPOCH) {
                                            Ok(n) => {
                                                if fetched_account.secret_timeout < n.as_secs() as u32 {
                                                    // validate that verified is true
                                                    if fetched_account.verified {
                                                        // validate that account owns the desired user_id
                                                        match User::belonging_to(&fetched_account).find(user_id)
                                                            .first::<User>(&connection) {
                                                                Ok(_) => {
                                                                    match Session::new(user_id) {
                                                                        Ok(session) => {
                                                                            // insert new session into db
                                                                            match diesel::insert_into(webe_sessions::table)
                                                                              .values(&session)
                                                                              .execute(&connection) {
                                                                                  Ok(_) => return Ok(session),
                                                                                  Err(err) => return Err(WebeAuthError::DBError(err))
                                                                            }
                                                                        },
                                                                        Err(err) => return Err(WebeAuthError::Session(err))
                                                                    }
                                                                },
                                                                Err(err) => return Err(WebeAuthError::DBError(err))
                                                        }
                                                    } else { return Err(WebeAuthError::NotVerifiedError); }
                                                } else { return Err(WebeAuthError::PasswordExpired); }
                                            },
                                            Err(_) => return Err(WebeAuthError::OtherError)
                                        }
                                    } else { return Err(WebeAuthError::LoginError); }// password doesn't match
                                },
                                Err(_) => return Err(WebeAuthError::LoginError) // bcrypt error
                            }
                        },
                        Err(_) => return Err(WebeAuthError::LoginError)
                }
            },
            Err(err) => return Err(WebeAuthError::PoolError(err))
        }
    }
}