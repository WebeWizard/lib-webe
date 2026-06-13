// NOTE: This test was written against an older, synchronous version of webe_web::server.
// The current server API is async (tokio), so the test body needs to be rewritten
// for the async API before it can run.
#![allow(unused_imports)]

use std::net::Ipv4Addr;

use webe::web::responders::file::FileResponder;
use webe::web::server::{Route, Server};

#[test]
#[ignore = "needs rewrite for async server API"]
fn start_server() {
    let ip = Ipv4Addr::new(127, 0, 0, 1);
    let port: u16 = 8080;

    // TODO: rewrite using:
    //   let server = Server::new(&ip, &port).await.expect("...");
    //   let mut route_map = RouteMap::new();
    //   ... add routes ...
    //   server.start(route_map).await;
    let _ = (ip, port);
}
