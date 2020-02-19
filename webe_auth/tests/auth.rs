extern crate dotenv;
extern crate webe_id;

use std::env;
use std::time::{Duration, SystemTime};

use webe_auth::{AuthError, AuthManager};

// TODO:  Probably best to split these into separate test modules

#[test]
fn auth_test() {
  dotenv::dotenv().unwrap();

  // create the email pool
  print!("Building Email Connection pool......");
  let smtp_address = env::var("SMTP_ADDRESS").expect("Failed to load SMTP Address from .env");
  let smtp_user = env::var("SMTP_USER").expect("Failed to load SMTP User from .env");
  let smtp_pass = env::var("SMTP_PASS").expect("Failed to load SMTP Password from .env");
  let email_pool = webe_auth::email::create_smtp_pool(smtp_address, smtp_user, smtp_pass)
    .expect("Failed to create SMTP pool");
  println!("Done");

  // create the database pool
  print!("Building Database Connection Pool......");
  let db_connect_string =
    env::var("AUTH_DATABASE_URL").expect("Failed to load DB Connect string from .env");
  let db_pool = webe_auth::db::new_manager(db_connect_string)
    .expect("Failed to create Database connection pool");
  println!("Done");

  // prepare the WebeID factory
  let epoch = SystemTime::UNIX_EPOCH
    .checked_add(Duration::from_millis(1546300800000)) // 01-01-2019 12:00:00 AM GMT
    .expect("failed to create custom epoch");
  let factory = webe_id::WebeIDFactory::new(epoch, 0u8).expect("Failed to create ID factory");

  // create the auth manager
  let auth_manager = webe_auth::WebeAuth {
    db_manager: db_pool,
    email_manager: email_pool,
    id_factory: &std::sync::Mutex::new(factory),
  };

  let email = "WebeAuthTest@gmail.com".to_owned();
  let pass = "test123".to_owned();

  // if the email is in use, delete it (cleanup from previous test)
  if let Ok(existing) = auth_manager.find_by_email(&email) {
    auth_manager.delete_account(existing).unwrap();
  }

  // CREATE the account
  let account = auth_manager
    .create_account(email.clone(), pass.clone())
    .unwrap();

  // READ it's actually saved in db
  let db_account = auth_manager.find_by_email(&email).unwrap();
  assert_eq!(&account.id, &db_account.id);

  // verify db email is lowercase
  assert_eq!(db_account.email, email.to_lowercase());

  // make sure account can't log in because un-verified
  match auth_manager.login(&email, &pass) {
    Ok(_) => panic!("Was able to log into unverified account"),
    Err(error) => match error {
      AuthError::AccountError(account_error) => match account_error {
        webe_auth::account::AccountError::NotVerified => {} // success
        _ => panic!("Error when attempting login when unverified"),
      },
      _ => panic!("Error when attempting login when unverified"),
    },
  }

  // UPDATE the account's verification
  auth_manager
    .verify_account(&email, &pass, &account.verify_code.unwrap())
    .unwrap();

  // verify db account is now verified
  let db_account = auth_manager.find_by_email(&email).unwrap();
  assert_eq!(db_account.verify_code, None);

  // make sure account can't log in because of bad pass
  match auth_manager.login(&email, &"321tset".to_owned()) {
    Ok(_) => panic!("Was able to log into account with bad pass"),
    Err(error) => match error {
      AuthError::AccountError(account_error) => match account_error {
        webe_auth::account::AccountError::BadPass => {} // success
        _ => panic!("Error when attempting login with bad pass"),
      },
      _ => panic!("Error when attempting login with bad pass"),
    },
  }

  // make sure account can log in with correct pass
  let session = auth_manager.login(&email, &pass).unwrap();

  // verify session exists in database
  webe_auth::db::SessionApi::find(&auth_manager.db_manager, session.token.as_str()).unwrap();

  // logout the session
  auth_manager.logout(&session.token).unwrap();

  // verify session is removed from database
  match webe_auth::db::SessionApi::find(&auth_manager.db_manager, session.token.as_str()) {
    Ok(_) => panic!("Found a session that should have been removed"),
    Err(error) => {
      match error {
        webe_auth::db::DBApiError::NotFound => {} // success
        _ => panic!(format!(
          "Error when attempting to check session in db.: {:?}",
          error
        )),
      }
    }
  }

  // DELETE the account
  auth_manager.delete_account(db_account).unwrap();

  // verify account is deleted
  match auth_manager.find_by_email(&email) {
    Ok(_) => panic!("Account wasn't deleted"),
    Err(error) => match error {
      AuthError::DBError(db_error) => match db_error {
        webe_auth::db::DBApiError::NotFound => {} // success
        _ => panic!("Error when finding account from email"),
      },
      _ => panic!("Error when finding account from email"),
    },
  }
}
