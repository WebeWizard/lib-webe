#[macro_use]
extern crate diesel;
extern crate rand;
extern crate uuid;

pub mod auth;
pub mod constants;
pub mod schema;

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
    LoginError,
    Session(SessionError),
    User(UserError)
}

impl WebeAuth {
    pub fn new(database_url: &String) -> Result<WebeAuth,WebeAuthError> {
        match MysqlConnection::establish(database_url) {
            Ok(connection) => Ok(WebeAuth { connection: connection }),
            Err(err) => return Err(WebeAuthError::DBConnError(err))
        }
    }

    pub fn create_account(&self, name: String, email: String, password: String) -> Result<Account,WebeAuthError> {
        // create the new account
        match Account::new(email, password) {
            Ok(new_account) => {
                // attempt to insert into database
                match diesel::insert_into(schema::webe_accounts::table)
                    .values(&new_account)
                    .execute(&self.connection) {
                        Ok(_) => {
                            match self.create_user(&new_account.id, name) {
                                Ok(_) => return Ok(new_account),
                                Err(err) => return Err(err)
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

    pub fn create_user(&self, account_id: &Vec<u8>, name: String) -> Result<User,WebeAuthError> {
        // create the new user
        match User::new(account_id, name) {
            Ok(new_user) => {
                // attempt to insert into database
                match diesel::insert_into(schema::webe_users::table)
                    .values(&new_user)
                    .execute(&self.connection) {
                        Ok(_) => return Ok(new_user),
                        Err(err) => return Err(WebeAuthError::DBError(err))
                }
            },
            Err(err) => return Err(WebeAuthError::User(err))
        }
    }

    pub fn login(&self, login_email: String, pass:String, user_id: &Vec<u8>) -> Result<Session,WebeAuthError> {
        // fetch account from db using email
        webe_accounts.filter()
        // - verify password and secret with bcrypt
        // validate that the secret_timeout for this pass has not expired
        // validate that verified is true
        // validate that account owns the desired user_id
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn auth_account_db() {
        assert_eq!(2 + 2, 4);
    }
}
