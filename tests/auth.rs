extern crate dotenv;

use lib_webe::auth::{WebeAuth};

// TODO:  Probably best to split these into separate test modules

#[test]
fn account_crud() {
  dotenv::dotenv().unwrap();
  let database_url = std::env::var("DATABASE_URL").unwrap();
  let auth_manager: WebeAuth = WebeAuth::new(&database_url).unwrap();

  let email = "WebeWizardAccountTest@gmail.com";

  // if the email is in use, delete it (cleanup from previous test)
  match auth_manager.get_account_by_email(&email.to_owned()) {
    Ok(old_account) => auth_manager.delete_account(&old_account.id).unwrap(),
    Err(_) => {} // ignore the error, we'll catch 'select' errors later
  }

  // CREATE the account
  let account = auth_manager.create_account(
    email.to_owned(),
    "test123".to_owned()
  ).unwrap();

  // READ it's actually saved in db
  let db_account = auth_manager.get_account_by_email(&email.to_owned()).unwrap();
  assert_eq!(account.id, db_account.id);

  // verify db email is lowercase
  assert_eq!(db_account.email, email.to_lowercase());

  // UPDATE the account's verification
  auth_manager.verify_account(&account.verify_code.unwrap()).unwrap();

  // DELETE the account
  auth_manager.delete_account(&account.id).unwrap();
}

#[test]
fn user_crud() {
  dotenv::dotenv().unwrap();
  let database_url = std::env::var("DATABASE_URL").unwrap();
  let auth_manager: WebeAuth = WebeAuth::new(&database_url).unwrap();

  let email = "WebeWizardUserTest@gmail.com".to_owned();

  // if the email is in use, delete it (cleanup from previous test)
  match auth_manager.get_account_by_email(&email.to_owned()) {
    Ok(old_account) => auth_manager.delete_account(&old_account.id).unwrap(),
    Err(_) => {} // ignore the error, we'll catch 'select' errors later
  }
  
  // create an account for the users to live under
  let account = auth_manager.create_account(
    email.to_owned(),
    "test123".to_owned()
  ).unwrap();

  // READ the initial user created with the account
  let account_user = auth_manager.get_user_by_name(&account.id, &"WebeWizard".to_owned()).unwrap();
  assert_eq!(account_user.name, "WebeWizard".to_owned());

  // CREATE a second new user under the same account
  let new_user = auth_manager.create_user(&account.id, "Meowness".to_owned()).unwrap();

  // TODO:  UPDATE

  // DELETE it
  auth_manager.delete_user(&new_user.id).unwrap();
  auth_manager.delete_account(&account.id).unwrap();
}

#[test]
fn session_crud() {
  dotenv::dotenv().unwrap();
  let database_url = std::env::var("DATABASE_URL").unwrap();
  let auth_manager: WebeAuth = WebeAuth::new(&database_url).unwrap();

  let email = "WebeWizardSessionTest@gmail.com";

  // if the email is in use, delete it (cleanup from previous test)
  match auth_manager.get_account_by_email(&email.to_owned()) {
    Ok(old_account) => auth_manager.delete_account(&old_account.id).unwrap(),
    Err(_) => {} // ignore the error, we'll catch 'select' errors later
  }
  
  // CREATE session from account login
  let account = auth_manager.create_account(
    email.to_owned(),
    "test123".to_owned()
  ).unwrap();
  
  // login should fail because account isn't verified yet
  match auth_manager.login(&account.email, &"test123".to_owned()) {
    Ok(_) => panic!("Login should fail if account is un-verified"),
    Err(_) => {}
  }
  
  // verify the account and try to log in again
  auth_manager.verify_account(&account.verify_code.unwrap()).unwrap();

  let session = auth_manager.login(&account.email, &"test123".to_owned()).unwrap();

  // READ session, make sure it's still valid
  let db_session = auth_manager.get_session(&session.token).unwrap();
  assert_eq!(db_session.is_expired().unwrap(), false);

  // UPDATE session with a selected user
  let user = auth_manager.get_user_by_name(&account.id, &"WebeWizard".to_owned()).unwrap();
  auth_manager.change_user(&session.token, &user.id).unwrap();

  // DELETE the session
  auth_manager.delete_session(&session.token).unwrap();
  auth_manager.delete_account(&account.id).unwrap();
}