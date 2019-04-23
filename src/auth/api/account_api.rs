use diesel::prelude::*;
use diesel::r2d2::{ManageConnection, PooledConnection, ConnectionManager};
use diesel::result::{DatabaseErrorKind, Error as DieselError};

use super::schema::{webe_accounts, webe_users, webe_sessions};
use super::schema::webe_accounts::dsl::*;

use super::account_model::{Account, AccountError};
use super::user_model::{User, UserError};

#[derive(Debug)]
pub enum AccountApiError {
    AlreadyExists, // account with given email address already exists
    AccountError(AccountError), // errors from Account model
    DBError(DieselError), // errors from interacting with database
    InvalidEmail, // email address does not meet validation standards
    InvalidPassword, // password does not meet validation standards
    UserError(UserError), // errors from User model
    VerifyExpired, // Verifiction code has expired
    VerifyFailed, // Verification attempt failed.
    OtherError
}

pub fn create_account<T> (connection: &T, user_name: String, new_email: String, password: String) -> Result<Account,AccountApiError>
where T: diesel::Connection<Backend = diesel::mysql::Mysql> // TODO: make this backend generic
{
    // create the new account
    match Account::new(new_email, password) {
        Ok(new_account) => {
            match diesel::insert_into(webe_accounts::table) // using the ::table format here to avoid the 'use'
                .values(&new_account)
                .execute(connection) {
                    Ok(_) => {
                        match User::new(&new_account.id, user_name) {
                            Ok(new_user) => {
                                // attempt to insert default user into database
                                match diesel::insert_into(webe_users::table)
                                    .values(&new_user)
                                    .execute(connection) {
                                        Ok(_) => return Ok(new_account),
                                        Err(err) => return Err(AccountApiError::DBError(err))
                                }
                            },
                            Err(err) => return Err(AccountApiError::UserError(err))
                        }
                    },
                    Err(err) => {
                        // check for unique constraint (currently only on PK and email)
                        match err {
                            DieselError::DatabaseError(dberr, _) => {
                                match dberr {
                                    DatabaseErrorKind::UniqueViolation =>
                                        return Err(AccountApiError::AlreadyExists),
                                    _ => return Err(AccountApiError::DBError(err))
                                }
                            },
                            _ => return Err(AccountApiError::DBError(err))
                        }
                    }
            }
        },
        Err(error) => return Err(AccountApiError::AccountError(error))
    }
}

pub fn find_by_email<T> (connection: &T, email_address: &String) -> Result<Account,AccountApiError>
where T: diesel::Connection<Backend = diesel::mysql::Mysql> // TODO: make this backend generic
{
    match webe_accounts.filter(email.eq(email_address)).first(connection) {
        Ok(account) => return Ok(account),
        Err(err) => return Err(AccountApiError::DBError(err))
    }
}

pub fn delete_account<T> (connection: &T, account_id: &Vec<u8>) -> Result<(),AccountApiError>
where T: diesel::Connection<Backend = diesel::mysql::Mysql> // TODO: make this backend generic
{
    // deleting the account will cascade to all related tables
    match diesel::delete(webe_accounts::table.find(account_id))
        .execute(connection) {
            Ok(_) => return Ok(()),
            Err(err) => return Err(AccountApiError::DBError(err))
        }
}