extern crate dotenv;

use diesel::prelude::*;

use lib_webe::auth::WebeAuth;
use lib_webe::auth::models::*;
use lib_webe::schema::webe_accounts::dsl::*;

#[test]
fn create_account() {
  dotenv::dotenv().ok();
  let database_url = std::env::var("DATABASE_URL").unwrap();
  let auth_manager: WebeAuth = WebeAuth::new(&database_url).unwrap();
  let con_pool = &auth_manager.con_pool;

  // attempt to create account
  let user_name = "WebeWizard".to_owned();
  let new_email: String = "webewizard@gmail.com".to_owned();
  let password = "test1234!@#$".to_owned();
  let new_account = auth_manager.create_account(
    user_name,
    new_email,
    password
  ).unwrap();
  let account_id = &new_account.id;

  // query the database for the created account and default user
  let connection = con_pool.get().unwrap();
  let fetched_account: account::Account = webe_accounts.find(account_id)
    .first(&connection).unwrap();
  assert_eq!(fetched_account.id, new_account.id)
}