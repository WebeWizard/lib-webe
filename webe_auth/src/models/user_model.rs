use super::account_model::Account;
use crate::schema::webe_users;

use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable, Insertable, Associations)]
#[belongs_to(Account, foreign_key = "account_id")]
#[table_name = "webe_users"]
pub struct User {
  pub id: Vec<u8>,
  pub account_id: Vec<u8>,
  pub name: String,
}

#[derive(Debug)]
pub enum UserError {}

impl User {
  pub fn new(account_id: &Vec<u8>, name: String) -> Result<User, UserError> {
    return Ok(User {
      id: Uuid::new_v4().as_bytes().to_vec(),
      account_id: account_id.to_vec(),
      name: name,
    });
  }
}
