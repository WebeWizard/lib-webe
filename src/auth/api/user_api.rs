use diesel::prelude::*;
use diesel::result::{Error as DieselError};

use super::schema::webe_users;
use super::schema::webe_users::dsl::*;

use super::user_model::{User, UserError};

#[derive(Debug)]
pub enum UserApiError {
    DBError(DieselError), // errors from interacting with database
    UserError(UserError), // errors from User model
    OtherError
}

pub fn create_user<T> (connection: &T, acc_id: &Vec<u8>, user_name: String) -> Result<User,UserApiError>
where T: diesel::Connection<Backend = diesel::mysql::Mysql> // TODO: make this backend generic
{  
  match User::new(acc_id, user_name) {
    Ok(new_user) => {
        // attempt to insert default user into database
        match diesel::insert_into(webe_users::table)
            .values(&new_user)
            .execute(connection) {
                Ok(_) => return Ok(new_user),
                Err(err) => return Err(UserApiError::DBError(err))
        }
    },
    Err(err) => return Err(UserApiError::UserError(err))
  }
}

pub fn find_by_name<T> (connection: &T, acc_id: &Vec<u8>, user_name: &String) -> Result<User,UserApiError>
where T: diesel::Connection<Backend = diesel::mysql::Mysql> // TODO: make this backend generic
{  
  match webe_users.filter(
        account_id.eq(acc_id)
        .and(name.eq(user_name))
      ).first(connection) {
      Ok(account) => return Ok(account),
      Err(err) => return Err(UserApiError::DBError(err))
  }
}

pub fn delete_user<T> (connection: &T, user_id: &Vec<u8>) -> Result<(),UserApiError>
where T: diesel::Connection<Backend = diesel::mysql::Mysql> // TODO: make this backend generic
{  
    match diesel::delete(webe_users::table.find(user_id))
      .execute(connection) {
          Ok(_) => return Ok(()),
          Err(err) => return Err(UserApiError::DBError(err))
      }
}