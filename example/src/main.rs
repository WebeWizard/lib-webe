extern crate dotenv;

extern crate webe_web;

use std::env;
use std::net::Ipv4Addr;

use webe_web::responders::static_message::StaticResponder;
use webe_web::responders::{file::FileResponder, options::OptionsResponder};
use webe_web::server::{Route, RouteMap, Server};

#[tokio::main]
async fn main() {
    // load environment
    print!("Loading Environment Config......");
    dotenv::dotenv().expect("Failed to load environment config file");
    println!("Done");

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
    let web_server = Server::new(&ip, &port)
        .await
        .expect("Failed to create web server");

    // add routes
    let mut route_map = RouteMap::new();
    // -- OPTIONS for preflight request
    let options_route = Route::new("OPTIONS", "/<dump>");
    let options_responder = OptionsResponder::new(
        "http://localhost:1234".to_owned(),
        "POST, GET, OPTIONS".to_owned(),
        "content-type, x-webe-token".to_owned(),
    );
    route_map.add_route(options_route, options_responder);

    // -- hello world
    let root_route = Route::new("GET", "/");
    let static_responder = StaticResponder::new(200, "Hello World".to_string());
    route_map.add_route(root_route, static_responder);

    // -- get static files
    let file_route = Route::new("GET", "/<path>");
    let file_responder = FileResponder::new(".".to_owned(), "<path>".to_owned())
        .expect("Failed to create FileResponder");
    route_map.add_route(file_route, file_responder);

    // -- test putting files
    let file_put_route = Route::new("PUT", "/<path>");
    let file_put_responder = FileResponder::new(".".to_owned(), "<path>".to_owned())
        .expect("Failed to create FileResponder");
    route_map.add_route(file_put_route, file_put_responder);

    println!("Done");

    // start the server
    println!("Running the server...");
    let _start_result = web_server.start(route_map).await;
}
