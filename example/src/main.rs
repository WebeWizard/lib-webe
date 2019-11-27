extern crate dotenv;

extern crate webe_auth;
extern crate webe_id;
extern crate webe_web;

use std::collections::HashMap;
use std::env;
use std::net::Ipv4Addr;
use std::time::{Duration, SystemTime};

use webe_auth::http::{create_account, login, logout, verify_account};
use webe_web::request::Request;
use webe_web::responders::{file::FileResponder, options::OptionsResponder, Responder};
use webe_web::response::Response;
use webe_web::server::{Route, Server};
use webe_web::validation::Validation;

fn main() {
  // load environment
  print!("Loading Environment Config......");
  dotenv::dotenv().expect("Failed to load environment config file");
  println!("Done");

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
  let db_manager = webe_auth::db::new_manager(db_connect_string)
    .expect("Failed to create Database connection pool");
  println!("Done");

  // create the unique ID factory
  let node_id = 0u8;
  let epoch = SystemTime::UNIX_EPOCH
    .checked_add(Duration::from_millis(1546300800000)) // 01-01-2019 12:00:00 AM GMT
    .expect("failed to create custom epoch");
  let id_factory = std::sync::Mutex::new(
    webe_id::WebeIDFactory::new(epoch, node_id).expect("Failed to create ID generator"),
  );

  // create the auth manager
  let auth_manager = webe_auth::WebeAuth {
    db_manager: db_manager,
    email_manager: email_pool,
    id_factory: &id_factory,
  };

  // create the web server
  print!("Setting up Web Server and Routes......");
  let web_bind_ip = env::var("WEB_BIND_IP").expect("Failed to load Web Server Bind IP from .env");
  let web_bind_port =
    env::var("WEB_BIND_PORT").expect("Failed to load Web Server Bind PORT from .env");
  let ip = web_bind_ip
    .parse::<Ipv4Addr>()
    .expect("Failed to parse WEB_BIND_IP as Ipv4Addr");
  let port = web_bind_port
    .parse::<u16>()
    .expect("Failed to parse WEB_BIND_PORT as u16");
  let mut web_server = Server::new(&ip, &port).expect("Failed to create web server");

  // add routes
  // -- OPTIONS for preflight request
  let options_route = Route::new("OPTIONS", "/<dump>");
  let options_responder = OptionsResponder::new(
    "http://localhost:1234".to_owned(),
    "POST, GET, OPTIONS".to_owned(),
    "content-type".to_owned(),
  );
  web_server.add_route(options_route, options_responder);

  // -- static files
  let file_route = Route::new("GET", "/<path>");
  let file_responder = FileResponder::new(".".to_owned(), "<path>".to_owned())
    .expect("Failed to create FileResponder");
  web_server.add_route(file_route, file_responder);

  // -- auth
  // -- -- account
  let create_account_route = Route::new("POST", "/account/create");
  let create_account_responder = create_account::CreateAccountResponder::new(&auth_manager);
  web_server.add_route(create_account_route, create_account_responder);

  let verify_account_route = Route::new("POST", "/account/verify");
  let verify_account_responder = verify_account::VerifyAccountResponder::new(&auth_manager);
  web_server.add_route(verify_account_route, verify_account_responder);

  // -- -- session
  let login_route = Route::new("POST", "/login");
  let login_responder = login::LoginResponder::new(&auth_manager);
  web_server.add_route(login_route, login_responder);

  let logout_route = Route::new("POST", "/logout");
  let logout_responder = logout::LogoutResponder::new(&auth_manager);
  web_server.add_route(logout_route, logout_responder);

  println!("Done");
  // start the server
  let _start_result = web_server.start();
}
