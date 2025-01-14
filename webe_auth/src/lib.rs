#[macro_use]
extern crate diesel;
extern crate lettre;
extern crate lettre_email;
extern crate native_tls;
extern crate r2d2;
extern crate rand;
extern crate serde;
extern crate webe_id;
extern crate webe_web;

/*
  Accounts - used to authenticate a single user
  Session - A result of logging in.
   - Contains a token for accessing account specific data
   - Contains an expiration date for timing out and forcing re-auth

  TODO - for every db table, have two mysql users, neither of which can edit table structure/permission
   - one that can insert/update/select/delete
   - one that can only read
*/

pub mod account;
pub mod constants;
pub mod db;
pub mod email;
pub mod http;
pub mod schema;
pub mod session;
pub mod utility;

use lettre_email::Email;
use std::sync::Arc;
use std::sync::Mutex;

use account::Account;
use db::DBApiError;
use email::{EmailApi, EmailError};
use session::{Session, SessionError};

#[derive(Debug)]
pub enum AuthError {
    AccountError(account::AccountError),
    DBError(DBApiError),
    EmailError(EmailError),
    OtherError,
    SessionError(SessionError),
}

pub trait AuthManager {
    // *ACCOUNT*
    fn create_account(&self, username: String, password: String) -> Result<Account, AuthError>;

    fn find_by_email(&self, email_address: &String) -> Result<Account, AuthError>;
    fn reset_verification(&self, email_address: &String, pass: &String) -> Result<(), AuthError>;

    fn verify_account(
        &self,
        email_address: &String,
        pass: &String,
        code: &String,
    ) -> Result<Session, AuthError>;

    // TODO: Reset password

    fn delete_account(&self, account: Account) -> Result<(), AuthError>;

    // *SESSION*
    fn find_valid_session(&self, token: &String) -> Result<Session, AuthError>;

    fn login(&self, email_address: &String, pass: &String) -> Result<Session, AuthError>;

    fn logout(&self, token: &String) -> Result<(), AuthError>;
}

pub struct WebeAuth {
    pub db_manager: db::DBManager,
    pub email_manager: email::EmailManager,
    pub id_factory: Arc<Mutex<webe_id::WebeIDFactory>>,
}

impl WebeAuth {
    pub fn new_id(&self) -> Result<u64, AuthError> {
        match self.id_factory.lock() {
            Ok(mut factory) => match factory.next() {
                Ok(id) => return Ok(id),
                _ => return Err(AuthError::OtherError),
            },
            // TODO: find a way to make the lock not poisonable
            _ => return Err(AuthError::OtherError), // mutex is poisoned
        }
    }

    pub fn send_verify_email(&self, account: &Account) -> Result<(), AuthError> {
        match &account.verify_code {
            Some(code) => {
                let email_builder = Email::builder()
                    .to(("webewizard@gmail.com", "WebeWizard"))
                    .from(("admin@webewizard.com", "WebeWizard.com Registration"))
                    .subject("WebeWizard.com Account Verification")
                    .text(format!("Verify Code: {}", code));
                let _result = self.email_manager.send_email(email_builder)?;
                return Ok(());
            }
            None => return Err(AuthError::OtherError),
        }
    }
}

impl AuthManager for WebeAuth {
    // *ACCOUNT*
    fn create_account(&self, username: String, password: String) -> Result<Account, AuthError> {
        // create account model
        let id = self.new_id()?;
        let account = Account::new(id, username, password)?;
        // save to database
        db::AccountApi::insert(&self.db_manager, &account)?;
        // send verification email
        self.send_verify_email(&account)?;
        return Ok(account);
    }

    fn find_by_email(&self, email_address: &String) -> Result<Account, AuthError> {
        let account = db::AccountApi::find_by_email(&self.db_manager, email_address)?;
        return Ok(account);
    }

    fn reset_verification(&self, email_address: &String, pass: &String) -> Result<(), AuthError> {
        // generate a new code and timeout
        match utility::new_verify_code() {
            Ok(new_verify) => {
                // get account from db matching email
                let account = db::AccountApi::find_by_email(&self.db_manager, email_address)?;
                // check password
                account.check_pass(pass)?;
                // update code and timeout  database
                db::AccountApi::reset_verification(&self.db_manager, &account.id, new_verify)?;
                // get the updated account object from db
                // TODO: Maybe one day mariadb will support UPDATE...RETURNING syntax
                let updated_account = db::AccountApi::find(&self.db_manager, &account.id)?;
                // send verification email
                self.send_verify_email(&updated_account)?;
                return Ok(());
            }
            Err(_err) => return Err(AuthError::OtherError),
        }
    }

    fn verify_account(
        &self,
        email_address: &String,
        pass: &String,
        code: &String,
    ) -> Result<Session, AuthError> {
        // get account from db matching email
        let account = db::AccountApi::find_by_email(&self.db_manager, email_address)?;
        // check password
        account.check_pass(pass)?;
        // check verify code valid
        account.check_verify(Some(code))?;
        // complete verify using db
        db::AccountApi::verify(&self.db_manager, &account.id, code)?;
        // create a new session for the account
        let session = Session::new(&account.id)?;
        // save session to db
        db::SessionApi::insert(&self.db_manager, &session)?;
        return Ok(session);
    }

    // TODO: Reset password

    // TODO: should this just take the account id? or a valid session?
    fn delete_account(&self, account: Account) -> Result<(), AuthError> {
        db::AccountApi::delete(&self.db_manager, account)?;
        return Ok(());
    }

    // *SESSION*
    fn find_valid_session(&self, token: &String) -> Result<Session, AuthError> {
        let session = db::SessionApi::find(&self.db_manager, token.as_str())?;
        if session.is_expired() {
            return Err(AuthError::SessionError(SessionError::SessionExpired));
        } else {
            return Ok(session);
        }
    }

    fn login(&self, email_address: &String, pass: &String) -> Result<Session, AuthError> {
        // get account from db matching email
        let account = db::AccountApi::find_by_email(&self.db_manager, email_address)?;
        // check password
        account.check_pass(pass)?;
        // check account is verified
        account.check_verify(None)?;
        // create a new session for the account
        let session = Session::new(&account.id)?;
        // save session to db
        db::SessionApi::insert(&self.db_manager, &session)?;
        return Ok(session);
    }

    fn logout(&self, token: &String) -> Result<(), AuthError> {
        // delete the session from the database
        db::SessionApi::delete(&self.db_manager, token.as_str())?;
        return Ok(());
    }
}
