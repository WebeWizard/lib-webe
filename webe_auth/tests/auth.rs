extern crate dotenv;
extern crate webe_id;

use std::env;
use std::time::{Duration, SystemTime};

use webe_auth::{AuthManager, WebeAuth};

// TODO:  Probably best to split these into separate test modules

#[test]
fn account_crud() {
  dotenv::dotenv().unwrap();
  let database_url = std::env::var("DATABASE_URL").unwrap();
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
    env::var("DATABASE_URL").expect("Failed to load DB Connect string from .env");
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

  let email = "WebeWizardAccountTest@gmail.com";

  // if the email is in use, delete it (cleanup from previous test)
  // match auth_manager.get_account_by_email(&email.to_owned()) {
  //   Ok(old_account) => auth_manager.delete_account(&old_account.id).unwrap(),
  //   Err(_) => {} // ignore the error, we'll catch 'select' errors later
  // }

  // CREATE the account
  let account = auth_manager
    .create_account(email.to_owned(), "test123".to_owned())
    .unwrap();

  // READ it's actually saved in db
  //   let db_account = auth_manager
  //     .get_account_by_email(&email.to_owned())
  //     .unwrap();
  //   assert_eq!(account.id, db_account.id);

  //   // verify db email is lowercase
  //   assert_eq!(db_account.email, email.to_lowercase());

  //   // UPDATE the account's verification
  //   auth_manager
  //     .verify_account(&account.verify_code.unwrap())
  //     .unwrap();

  //   // DELETE the account
  //   auth_manager.delete_account(&account.id).unwrap();
}

// #[test]
// fn session_crud() {
//   dotenv::dotenv().unwrap();
//   let database_url = std::env::var("DATABASE_URL").unwrap();

//   // create the email pool
//   print!("Building Email Connection pool......");
//   let smtp_address = env::var("SMTP_ADDRESS").expect("Failed to load SMTP Address from .env");
//   let smtp_user = env::var("SMTP_USER").expect("Failed to load SMTP User from .env");
//   let smtp_pass = env::var("SMTP_PASS").expect("Failed to load SMTP Password from .env");
//   let email_pool = webe_auth::email::create_smtp_pool(smtp_address, smtp_user, smtp_pass)
//     .expect("Failed to create SMTP pool");
//   println!("Done");

//   // create the database pool
//   print!("Building Database Connection Pool......");
//   let db_connect_string =
//     env::var("DATABASE_URL").expect("Failed to load DB Connect string from .env");
//   let db_pool = webe_auth::database::create_db_pool(db_connect_string)
//     .expect("Failed to create Database connection pool");
//   println!("Done");

//   // create the auth manager
//   let auth_manager = webe_auth::WebeAuth {
//     db_conn_pool: db_pool,
//     email_conn_pool: email_pool,
//   };

//   let email = "WebeWizardSessionTest@gmail.com";

//   // if the email is in use, delete it (cleanup from previous test)
//   match auth_manager.get_account_by_email(&email.to_owned()) {
//     Ok(old_account) => auth_manager.delete_account(&old_account.id).unwrap(),
//     Err(_) => {} // ignore the error, we'll catch 'select' errors later
//   }
//   // CREATE session from account login
//   let account = auth_manager
//     .create_account(email.to_owned(), "test123".to_owned())
//     .unwrap();
//   // login should fail because account isn't verified yet
//   match auth_manager.login(&account.email, &"test123".to_owned()) {
//     Ok(_) => panic!("Login should fail if account is un-verified"),
//     Err(_) => {}
//   }
//   // verify the account and try to log in again
//   auth_manager
//     .verify_account(&account.verify_code.unwrap())
//     .unwrap();

//   let session = auth_manager
//     .login(&account.email, &"test123".to_owned())
//     .unwrap();

//   // READ session, make sure it's still valid
//   let db_session = auth_manager.get_session(&session.token).unwrap();
//   assert_eq!(db_session.is_expired().unwrap(), false);

//   // DELETE the session
//   auth_manager.delete_session(&session.token).unwrap();
//   auth_manager.delete_account(&account.id).unwrap();
// }
