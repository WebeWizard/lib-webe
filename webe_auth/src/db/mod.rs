// This module contains database CRUD operations for each of the models.

use diesel::prelude::*;
use diesel::r2d2 as diesel_r2d2;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::result::{DatabaseErrorKind, Error as DieselError};

use crate::account::Account;
use crate::schema::webe_accounts::dsl::*;
use crate::schema::webe_sessions::dsl::*;
use crate::session::Session;

#[derive(Debug)]
pub enum DBApiError {
  AlreadyExists,           // account with given email address already exists
  OtherError(DieselError), // errors from interacting with database
  BadVerifyCode,
  PoolError(r2d2::Error),
  NotFound,
}

impl From<DBApiError> for crate::AuthError {
  fn from(err: DBApiError) -> crate::AuthError {
    crate::AuthError::DBError(err)
  }
}

impl From<r2d2::Error> for DBApiError {
  fn from(err: r2d2::Error) -> DBApiError {
    DBApiError::PoolError(err)
  }
}

impl From<DieselError> for DBApiError {
  fn from(err: DieselError) -> DBApiError {
    DBApiError::OtherError(err)
  }
}

pub type DBManager = diesel_r2d2::Pool<diesel_r2d2::ConnectionManager<MysqlConnection>>;

pub fn new_manager(connect_string: String) -> Result<DBManager, DBApiError> {
  let connection_manager = ConnectionManager::new(connect_string.as_str());
  // build the database connection pool
  let pool = Pool::builder().max_size(10).build(connection_manager)?;
  return Ok(pool);
}

pub trait AccountApi {
  fn insert(&self, account: &Account) -> Result<(), DBApiError>;

  fn find(&self, id: &u64) -> Result<Account, DBApiError>;

  fn find_by_email(&self, email: &String) -> Result<Account, DBApiError>;

  fn verify(&self, id: &u64, code: &String) -> Result<(), DBApiError>;

  fn reset_verification(&self, id: &u64, new_verify: (String, u32)) -> Result<(), DBApiError>;

  // TODO: need way to update email and password

  fn delete(&self, account: Account) -> Result<(), DBApiError>;
}

impl AccountApi for DBManager {
  fn insert(&self, account: &Account) -> Result<(), DBApiError> {
    let conn = self.get()?;
    match diesel::insert_into(webe_accounts)
      .values(account)
      .execute(&conn)
    {
      Ok(_) => return Ok(()),
      Err(err) => {
        // check for unique constraint (currently only on PK and email)
        match err {
          DieselError::DatabaseError(dberr, _) => match dberr {
            DatabaseErrorKind::UniqueViolation => return Err(DBApiError::AlreadyExists),
            _ => return Err(DBApiError::OtherError(err)),
          },
          _ => return Err(DBApiError::OtherError(err)),
        }
      }
    }
  }

  fn find(&self, acc_id: &u64) -> Result<Account, DBApiError> {
    let conn = self.get()?;
    let account = webe_accounts.find(acc_id).first(&conn)?;
    return Ok(account);
  }

  fn find_by_email(&self, account_email: &String) -> Result<Account, DBApiError> {
    let conn = self.get()?;
    let account = webe_accounts.filter(email.eq(account_email)).first(&conn)?;
    return Ok(account);
  }

  fn verify(&self, acc_id: &u64, code: &String) -> Result<(), DBApiError> {
    let conn = self.get()?;
    let result = diesel::update(
      webe_accounts
        .filter(id.eq(acc_id))
        .filter(verify_code.eq(code)),
    )
    .set((
      verify_code.eq(Option::<String>::None),
      verify_timeout.eq(Option::<u32>::None),
    ))
    .execute(&conn)?;
    if result == 0 {
      return Err(DBApiError::NotFound);
    };
    return Ok(());
  }

  fn reset_verification(&self, acc_id: &u64, new_verify: (String, u32)) -> Result<(), DBApiError> {
    let conn = self.get()?;
    let result = diesel::update(webe_accounts.find(acc_id))
      .set((
        verify_code.eq(new_verify.0),
        verify_timeout.eq(new_verify.1),
      ))
      .execute(&conn)?;
    if result == 0 {
      return Err(DBApiError::NotFound);
    };
    return Ok(());
  }

  // TODO: need way to update email and password

  fn delete(&self, account: Account) -> Result<(), DBApiError> {
    let conn = self.get()?;
    let result = diesel::delete(&account).execute(&conn)?;
    if result == 0 {
      return Err(DBApiError::NotFound);
    };
    return Ok(());
  }
}

pub trait SessionApi {
  fn insert(&self, session: &Session) -> Result<(), DBApiError>;

  fn delete(&self, token: &str) -> Result<(), DBApiError>;
}

impl SessionApi for DBManager {
  fn insert(&self, session: &Session) -> Result<(), DBApiError> {
    let conn = self.get()?;
    match diesel::insert_into(webe_sessions)
      .values(session)
      .execute(&conn)
    {
      Ok(_) => return Ok(()),
      Err(err) => {
        // check for unique constraint (currently only on Token)
        match err {
          DieselError::DatabaseError(dberr, _) => match dberr {
            DatabaseErrorKind::UniqueViolation => return Err(DBApiError::AlreadyExists),
            _ => return Err(DBApiError::OtherError(err)),
          },
          _ => return Err(DBApiError::OtherError(err)),
        }
      }
    }
  }

  fn delete(&self, session_token: &str) -> Result<(), DBApiError> {
    let conn = self.get()?;
    let result = diesel::delete(webe_sessions.filter(token.eq(session_token))).execute(&conn)?;
    if result == 1 {
      return Ok(());
    } else {
      return Err(DBApiError::NotFound);
    }
  }
}
