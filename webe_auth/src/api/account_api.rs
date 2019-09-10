use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};

use crate::schema::webe_accounts;
use crate::schema::webe_accounts::dsl::*;

use super::account_model::{Account, AccountError};

use crate::utility;

#[derive(Debug)]
pub enum AccountApiError {
    AlreadyExists,              // account with given email address already exists
    AccountError(AccountError), // errors from Account model
    DBError(DieselError),       // errors from interacting with database
    OtherError,
    BadVerifyCode,
}

pub fn find<T>(connection: &T, account_id: &Vec<u8>) -> Result<Account, AccountApiError>
where
    T: diesel::Connection<Backend = diesel::mysql::Mysql>, // TODO: make this backend generic
{
    match webe_accounts::table.find(account_id).first(connection) {
        Ok(account) => return Ok(account),
        Err(err) => return Err(AccountApiError::DBError(err)),
    }
}

pub fn create_account<T>(
    connection: &T,
    new_email: String,
    password: String,
) -> Result<Account, AccountApiError>
where
    T: diesel::Connection<Backend = diesel::mysql::Mysql>, // TODO: make this backend generic
{
    // create the new account
    match Account::new(new_email, password) {
        Ok(new_account) => {
            match diesel::insert_into(webe_accounts::table) // using the ::table format here to avoid the 'use'
                .values(&new_account)
                .execute(connection)
            {
                Ok(_) => return Ok(new_account),
                Err(err) => {
                    // check for unique constraint (currently only on PK and email)
                    match err {
                        DieselError::DatabaseError(dberr, _) => match dberr {
                            DatabaseErrorKind::UniqueViolation => {
                                return Err(AccountApiError::AlreadyExists)
                            }
                            _ => return Err(AccountApiError::DBError(err)),
                        },
                        _ => return Err(AccountApiError::DBError(err)),
                    }
                }
            }
        }
        Err(error) => return Err(AccountApiError::AccountError(error)),
    }
}

pub fn find_by_email<T>(connection: &T, email_address: &String) -> Result<Account, AccountApiError>
where
    T: diesel::Connection<Backend = diesel::mysql::Mysql>, // TODO: make this backend generic
{
    match webe_accounts
        .filter(email.eq(email_address))
        .first(connection)
    {
        Ok(account) => return Ok(account),
        Err(err) => return Err(AccountApiError::DBError(err)),
    }
}

pub fn verify<T>(connection: &T, code: &String) -> Result<(), AccountApiError>
where
    T: diesel::Connection<Backend = diesel::mysql::Mysql>,
{
    // TODO: filter by non-expired
    match diesel::update(webe_accounts.filter(verify_code.eq(code)))
        .set((
            verified.eq(true),
            verify_code.eq(None as Option<String>),
            verify_timeout.eq(None as Option<u32>),
        ))
        .execute(connection)
    {
        Ok(update_count) => {
            // TODO: assuming that no 2 accounts will have the same verify code
            // need to make sure the database sees these as unique
            if update_count == 0 {
                return Err(AccountApiError::BadVerifyCode);
            } else {
                return Ok(());
            }
        }
        Err(err) => return Err(AccountApiError::DBError(err)),
    }
}

// TODO:  be careful that this only gets called if account is not already verified
pub fn reset_verification<T>(connection: &T, account_id: &Vec<u8>) -> Result<(), AccountApiError>
where
    T: diesel::Connection<Backend = diesel::mysql::Mysql>,
{
    match utility::new_verify_code() {
        Ok(new_code) => {
            match diesel::update(webe_accounts.find(account_id))
                .set((verify_code.eq(new_code.0), verify_timeout.eq(new_code.1)))
                .execute(connection)
            {
                Ok(_) => return Ok(()),
                Err(err) => return Err(AccountApiError::DBError(err)),
            }
        }
        Err(_err) => return Err(AccountApiError::OtherError),
    }
}

pub fn delete_account<T>(connection: &T, account_id: &Vec<u8>) -> Result<(), AccountApiError>
where
    T: diesel::Connection<Backend = diesel::mysql::Mysql>, // TODO: make this backend generic
{
    // deleting the account will cascade to all related tables
    match diesel::delete(webe_accounts::table.find(account_id)).execute(connection) {
        Ok(_) => return Ok(()),
        Err(err) => return Err(AccountApiError::DBError(err)),
    }
}
