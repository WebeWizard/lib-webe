// NOTE: This test was written against an older, synchronous version of webe_web::server.
// The current server API is async (tokio), so the test body needs to be rewritten
// for the async API before it can run.
// Requires the `auth` feature to compile.
#![cfg(feature = "auth")]
#![allow(unused_imports)]

use webe::auth::WebeAuth;
use webe::auth::http::login::LoginResponder;
use webe::web::server::{Route, Server};

use std::net::Ipv4Addr;

#[test]
#[ignore = "needs rewrite for async server API"]
fn auth_server() {
    dotenvy::dotenv().ok();
    let ip = Ipv4Addr::new(127, 0, 0, 1);
    let port: u16 = 8080;

    let database_url = std::env::var("DATABASE_URL").unwrap();
    let auth_manager: WebeAuth = WebeAuth::new(&database_url).unwrap();

    let email = "WebeWizardSessionTest@gmail.com";
    match auth_manager.get_account_by_email(&email.to_owned()) {
        Ok(old_account) => auth_manager.delete_account(&old_account.id).unwrap(),
        Err(_) => {}
    }
    let account = auth_manager
        .create_account(
            "WebeWizard".to_owned(),
            email.to_owned(),
            "test123".to_owned(),
        )
        .unwrap();

    auth_manager
        .verify_account(&account.verify_code.unwrap())
        .unwrap();

    // TODO: rewrite using async Server::new(..).await and start() API
    auth_manager.delete_account(&account.id).unwrap();
}
