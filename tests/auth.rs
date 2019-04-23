extern crate dotenv;

use lib_webe::auth::WebeAuth;

#[test]
fn account_CRUD() {
  dotenv::dotenv().unwrap();
  let database_url = std::env::var("DATABASE_URL").unwrap();
  let auth_manager: WebeAuth = WebeAuth::new(&database_url).unwrap();

  // if the email is in use, delete it (cleanup from previous test)
  match auth_manager.select_account_by_email(&"webewizard@gmail.com".to_owned()) {
    Ok(old_account) => auth_manager.delete_account(old_account.id).unwrap(),
    Err(err) => {} // ignore the error, we'll catch 'select' errors later
  }

  // create the account
  let account = auth_manager.create_account(
    "WebeWizard".to_owned(),
    "WebeWizard@gmail.com".to_owned(),
    "test123".to_owned()
  ).unwrap();

  // verify it's actually saved in db
  let db_account = auth_manager.select_account_by_email(&"webewizard@gmail.com".to_owned()).unwrap();
  assert_eq!(account.id, db_account.id);
  // verify db email is lowercase
  assert_eq!(db_account.email, "webewizard@gmail.com");

  // TODO: UPDATE

  // delete the account
  auth_manager.delete_account(account.id).unwrap();
}