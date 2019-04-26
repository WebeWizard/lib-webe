extern crate dotenv;

use lib_webe::auth::WebeAuth;

// TODO:  Probably best to split these into separate test modules

#[test]
fn account_crud() {
  dotenv::dotenv().unwrap();
  let database_url = std::env::var("DATABASE_URL").unwrap();
  let auth_manager: WebeAuth = WebeAuth::new(&database_url).unwrap();

  // if the email is in use, delete it (cleanup from previous test)
  match auth_manager.select_account_by_email(&"webewizard@gmail.com".to_owned()) {
    Ok(old_account) => auth_manager.delete_account(&old_account.id).unwrap(),
    Err(_) => {} // ignore the error, we'll catch 'select' errors later
  }

  // CREATE the account
  let account = auth_manager.create_account(
    "WebeWizard".to_owned(),
    "WebeWizard@gmail.com".to_owned(),
    "test123".to_owned()
  ).unwrap();

  // READ it's actually saved in db
  let db_account = auth_manager.select_account_by_email(&"webewizard@gmail.com".to_owned()).unwrap();
  assert_eq!(account.id, db_account.id);

  // verify db email is lowercase
  assert_eq!(db_account.email, "webewizard@gmail.com");

  // UPDATE the account's verification
  auth_manager.reset_verification(&account.id).unwrap();

  // DELETE the account
  auth_manager.delete_account(&account.id).unwrap();
}

#[test]
fn user_crud() {
  
}