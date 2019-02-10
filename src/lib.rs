#[macro_use]
extern crate diesel;
extern crate rand;
extern crate uuid;

pub mod auth;
pub mod constants;
pub mod schema;

use std::time::{SystemTime, UNIX_EPOCH};

use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};

use schema::webe_accounts;
use schema::webe_accounts::dsl::*;
use auth::models::account::{Account, AccountError};
use auth::models::session::{Session, SessionError};
use auth::models::user::{User, UserError};

pub struct WebeAuth {
    connection: diesel::mysql::MysqlConnection
}

pub enum WebeAuthError {
    Account(AccountError),
    DBConnError(ConnectionError),
    DBError(DieselError),
    LoginError, // generic login error
    OtherError,
    PasswordExpired,
    Session(SessionError),
    SessionNotFound,
    SessionExpired,
    NotVerifiedError,
    User(UserError)
}

impl WebeAuth {
    pub fn new(database_url: &String) -> Result<WebeAuth,WebeAuthError> {
        match MysqlConnection::establish(database_url) {
            Ok(connection) => Ok(WebeAuth { connection: connection }),
            Err(err) => return Err(WebeAuthError::DBConnError(err))
        }
    }

    pub fn create_account(&self, user_name: String, new_email: String, password: String) -> Result<Account,WebeAuthError> {
        // create the new account
        match Account::new(new_email, password) {
            Ok(new_account) => {
                // attempt to insert new account into database
                match diesel::insert_into(schema::webe_accounts::table)
                    .values(&new_account)
                    .execute(&self.connection) {
                        Ok(_) => {
                            match User::new(&new_account.id, user_name) {
                                Ok(new_user) => {
                                    // attempt to insert default user into database
                                    match diesel::insert_into(schema::webe_users::table)
                                        .values(&new_user)
                                        .execute(&self.connection) {
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
            Err(error) => return Err(WebeAuthError::Account(error))
        }
    }
    
    // attempts to update the db session record with a new user and if successful, mutates the session
    pub fn change_user(&self, current_session: &mut Session, user_id: &Vec<u8>) -> Result<(),WebeAuthError> {
        match diesel::update(current_session as &Session) // diesel doesn't like mutable references
            .set(schema::webe_sessions::user_id.eq(user_id))
            .execute(&self.connection) {
                Ok(1) => { // only 1 record updated is a success
                    current_session.user_id = Some(user_id.to_vec());
                    return Ok(());
                },
                Ok(_) => return Err(WebeAuthError::SessionNotFound),
                Err(err) => return Err(WebeAuthError::DBError(err))
        }
    }

    pub fn login(&self, login_email: String, pass: String, user_id: &Vec<u8>) -> Result<Session,WebeAuthError> {
        // fetch account from db using email
        // TODO: check for multiple results instead of just taking 'first'
        match webe_accounts.filter(webe_accounts::email.eq(login_email))
            .first::<Account>(&self.connection) {
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
                                                    .first::<User>(&self.connection) {
                                                        Ok(_) => {
                                                            match Session::new(user_id) {
                                                                Ok(session) => {
                                                                    // insert new session into db
                                                                    match diesel::insert_into(schema::webe_sessions::table)
                                                                        .values(&session)
                                                                        .execute(&self.connection) {
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
                            } else {
                                // password doesn't match
                                return Err(WebeAuthError::LoginError);
                            }
                        },
                        Err(_) => return Err(WebeAuthError::LoginError) // bcrypt error
                    }
                },
                Err(_) => return Err(WebeAuthError::LoginError)
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn connect_webeauth_to_db() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn create_new_account() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn verify_account() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn log_into_account() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn change_user() {
        assert_eq!(2 + 2, 4);
    }
}
