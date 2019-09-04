extern crate dotenv;

use lib_webe::auth::http::login::LoginResponder;
use lib_webe::auth::WebeAuth;
use lib_webe::http::server::{Route, Server};

use std::net::Ipv4Addr;

#[test]
fn auth_server() {
  // initialize the server
  let ip = Ipv4Addr::new(127, 0, 0, 1);
  let port: u16 = 8080;

  // create the auth manager
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
  let account = auth_manager
    .create_account(
      "WebeWizard".to_owned(),
      email.to_owned(),
      "test123".to_owned(),
    )
    .unwrap();

  // verify the account and try to log in again
  auth_manager
    .verify_account(&account.verify_code.unwrap())
    .unwrap();

  match Server::new(&ip, &port) {
    Ok(mut server) => {
      // prepare the login responder and route
      let login_responder = LoginResponder::new(&auth_manager);
      let login_route = Route {
        method: "GET".to_owned(),
        uri: "/login".to_owned(),
      };
      // TODO: may fail if can't get mutable ref to routes arc
      server.add_route(login_route, login_responder);

      server.start();
    }
    Err(_) => panic!("Failed to create server"),
  }

  // DELETE the account
  auth_manager.delete_account(&account.id).unwrap();
}
