use std::boxed::Box;
use std::net::Ipv4Addr;
use std::sync::Arc;

use lib_webe::http::server::Server;
use lib_webe::http::responders::file::FileResponder;

#[test]
fn start_server() {
    // initialize the server
    let ip = Ipv4Addr::new(127,0,0,1);
    let port: u16 = 8080;
    match Server::new(&ip, &port) {
        Ok(mut server) => {
            match Arc::get_mut(&mut server.routes) {
                Some(routes) => {
                    // build a simple route
                    match FileResponder::new("/home/webe".to_owned(), "<path>".to_owned()) {
                        Ok(simple_file) => {
                            let route = ("GET".to_owned(),"/files/<path>".to_owned());
                            routes.insert(route, Box::new(simple_file));
                        },
                        Err(_error) => {
                            panic!("Bad path provided to FileResponer.");
                        }
                    }
                    // start the server
                    match server.start() {
                        Ok(()) => {},
                        Err(_error) => {
                            panic!("Server failed to start.");
                        }
                    }
                },
                None => panic!("Server routes can't be mutable")
            }
        },
        Err(_error) => {
            panic!("Failed to create server!");
        }
    }
}