use std::env;
use std::net::Ipv4Addr;
use std::process;

use webe_args::{OptionDef, OptionResult, Registry};
use webe_web::responders::static_message::StaticResponder;
use webe_web::responders::{file::FileResponder, options::OptionsResponder};
use webe_web::server::{Route, RouteMap, Server};

#[tokio::main]
async fn main() {
    // declare the command-line options
    print!("Parsing command-line options......");
    let mut registry = Registry::new();
    registry
        .add(
            OptionDef::value("bind-ip")
                .short("i")
                .description("IPv4 address for the web server to bind")
                .required()
                .validate(|v| v.parse::<Ipv4Addr>().is_ok()),
        )
        .add(
            OptionDef::value("bind-port")
                .short("p")
                .description("TCP port for the web server to bind")
                .required()
                .validate(|v| v.parse::<u16>().is_ok()),
        );

    let tokens: Vec<String> = env::args().skip(1).collect();

    // validate the whole command line up front
    let report = registry.validate(&tokens);
    if !report.is_success() {
        eprintln!("Failed");
        for failure in report.failures() {
            eprintln!("  {failure}");
        }
        eprintln!("usage: basic_server --bind-ip <IPv4> --bind-port <PORT>");
        process::exit(1);
    }
    println!("Done");

    // create the web server
    print!("Setting up Web Server and Routes......");
    let ip = match registry.read("bind-ip", &tokens) {
        Ok(OptionResult::Value(value)) => value
            .parse::<Ipv4Addr>()
            .expect("validated bind-ip should parse as Ipv4Addr"),
        _ => unreachable!("bind-ip is required and validated above"),
    };
    let port = match registry.read("bind-port", &tokens) {
        Ok(OptionResult::Value(value)) => value
            .parse::<u16>()
            .expect("validated bind-port should parse as u16"),
        _ => unreachable!("bind-port is required and validated above"),
    };
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
