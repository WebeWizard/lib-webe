use std::collections::HashMap;
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::io::{BufReader, BufWriter};
use std::time::Duration;
use std::sync::Arc;
use std::thread;

use super::request::Request;
use super::responders::{Responder};
use super::responders::static_message::StaticResponder;

#[derive(PartialEq,Eq,Hash)]
pub struct Route {
    pub method: String,
    pub uri: String
}

type RouteMap = HashMap<Route, Box<Responder>>;

pub struct Server {
    pub ip: Ipv4Addr,
    pub port: u16,
    pub routes: Arc<RouteMap>,
    listener: TcpListener,
}

pub enum ServerError {
    BindError(std::io::Error), // server failed to bind on ip and port
    ConnectionFailed(std::io::Error), // server failed to grab connection from listener
    InternalError, // failed to process the stream
}

impl Server {
    pub fn new(ip: &Ipv4Addr, port: &u16) -> Result<Server, ServerError> {
        // attempt to bind the server to the specified ip and port
        match TcpListener::bind((ip.clone(), port.clone())) {
            Ok(listener) => {
                return Ok(Server{ 
                    ip: ip.clone(),
                    port: port.clone(),
                    listener: listener,
                    routes: Arc::new(HashMap::<Route, Box<Responder>>::new())
                })
            },
            Err(error) => {return Err(ServerError::BindError(error))}
        };
    }

    pub fn add_route<T: Responder+'static>(&mut self, mut route: Route, responder: T) {
        match Arc::get_mut(&mut self.routes) { // TODO: not sure why this is necessary when already borrowing mut self
            Some(routes) => {
                // remove leading / if any
                if !route.uri.starts_with('/') { route.uri = "/".to_owned()+route.uri.as_str(); }
                routes.insert(route, Box::new(responder));
            },
            None => {} //TODO: should this return some error if we can't get the mutable ref?
        }
    }


    // starts the server, blocks the thread while the server is running
    pub fn start(&self) -> Result<(), ServerError> {
        println!("starting the server");
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let routes = self.routes.clone();
                    thread::spawn(move || {
                        loop {
                            match Server::process_stream(&stream, &routes) {
                                Ok(keep_alive) => if !keep_alive { break } 
                                Err(_) => { break } // server errors should break the connection (start fresh?)
                            }
                        }
                    });
                },
                Err(error) => return Err(ServerError::ConnectionFailed(error))
            }
        }
        return Ok(());
    }

    // process a client request, return a bool to keep connection alive
    // TODO: handle these errors better (need to know the real error for logging, whatever)
    pub fn process_stream(stream: &TcpStream, routes: &Arc<RouteMap>) -> Result<bool, ServerError> {
        // keep alive by default with 10 second timeout
        let mut keep_alive = true;
        match stream.set_read_timeout(Some(Duration::from_secs(10))) {
            Ok(_) => {
                // get the request from the stream
                let buf_reader = BufReader::new(stream);
                match Request::new(buf_reader) {
                    Ok(request) => {
                        println!("Processing Request. URI: {:?}",request.uri);
                        // TODO: determine if request needs decoded?

                        // use best route to respond to request
                        match Server::find_best_route(&request, routes){
                            Some(route) => {
                                match routes.get(route) {
                                    Some(responder) => {
                                        // process any route parameters
                                        let mut params = HashMap::<String,String>::new();
                                        let request_uri_parts: Vec<&str> = request.uri.split('/').collect();
                                        let route_uri_parts: Vec<&str> = route.uri.split('/').collect();
                                        let part_length = route_uri_parts.len();
                                        for i in 0..part_length {
                                            // if this is the last part of the route
                                            // and the part is a route param
                                            // then grab the remaining parts from the request uri
                                            if route_uri_parts[i].contains('<') {
                                                let name = route_uri_parts[i].to_owned();
                                                let value = if i==part_length-1 {
                                                        request_uri_parts[i..].join("/")
                                                    } else {
                                                        request_uri_parts[i].to_owned()
                                                    };
                                                params.insert(name, value);
                                            }
                                        }
                                            
                                        match responder.validate(&request, &params) {
                                            Ok(validation_code) => {
                                                match responder.build_response(&request, &params, validation_code) {
                                                    Ok(mut response) => {
                                                        match response.respond(BufWriter::new(&stream)) {
                                                            Ok(()) => keep_alive = response.keep_alive, 
                                                            Err(_error) => return Err(ServerError::InternalError)
                                                        }
                                                    },
                                                    Err(response_code) => {
                                                        let static_responder = StaticResponder::from_standard_code(response_code);
                                                        match static_responder.build_response(&request, &params, response_code) {
                                                            Ok(mut response) => {
                                                                match response.respond(BufWriter::new(&stream)) {
                                                                    Ok(()) => {}, // keep_alive = true
                                                                    Err(_error) => return Err(ServerError::InternalError)
                                                                }
                                                            },
                                                            Err(_error) => return Err(ServerError::InternalError)
                                                        }
                                                        
                                                    }
                                                }
                                            },
                                            Err(validation_code) => {
                                                let static_responder = StaticResponder::from_standard_code(validation_code);
                                                match static_responder.build_response(&request, &params, validation_code) {
                                                    Ok(mut response) => {
                                                        match response.respond(BufWriter::new(&stream)) {
                                                            Ok(()) => {}, // keep-alive = true
                                                            Err(_error) => return Err(ServerError::InternalError)
                                                        }
                                                    },
                                                    Err(_error) => return Err(ServerError::InternalError)
                                                }
                                            }
                                        }                                        
                                    },
                                    None => return Err(ServerError::InternalError)
                                }
                            },
                            None => {
                                let static_responder = StaticResponder::from_standard_code(400);
                                match static_responder.build_response(&request, &HashMap::<String,String>::new(), 400) {
                                    Ok(mut response) => {
                                        match response.respond(BufWriter::new(&stream)) {
                                            Ok(()) => {}, //keep-alive = true
                                            Err(_error) => return Err(ServerError::InternalError)
                                        }
                                    },
                                    Err(_error) => return Err(ServerError::InternalError)
                                }
                            } 
                        }
                    },
                    Err(_error) => return Err(ServerError::InternalError)
                }
            },
            Err(_) => return Err(ServerError::InternalError)
        }
        // TODO: keep the connection alive unless the *response* wants to kill it
        return Ok(keep_alive);
    }

    // TODO: 
    pub fn find_best_route<'a> (request: &Request, routes: &'a Arc<RouteMap>) -> Option<&'a Route> {
        // ~~ find the best responder ~~
        // non-terminal route params WILL NOT contain more than one request uri part
        // terminal route params WILL contain the remainder of the request uri
        let request_uri_parts: Vec<&str> = request.uri.split('/').collect();
        let request_uri_length = request_uri_parts.len();
        // only keys with matching method
        let keys: Vec<&Route> = routes.keys().filter(|key| key.method == request.method).collect();
        let mut matched = false;
        match keys.iter().max_by_key(|route| {
            let route_uri_parts: Vec<&str> = route.uri.split('/').collect();
            // compare length
            let route_length = route_uri_parts.len();
            if route_length > request_uri_length {return 0}
            // find the one with the most matching parts
            let mut match_size = 0;
            for part in &request_uri_parts {
                if part == &route_uri_parts[match_size] || route_uri_parts[match_size].contains('<') {
                    match_size += 1;
                    if match_size == route_length {break} // full match
                } else { // uri doesn't match
                    match_size = 0; 
                    break
                }
            }
            if match_size > 0 {matched = true} // use this to determine if no routes found
            return match_size;
        }) {
            Some(key) => {
                if !matched {return None};
                return Some(key)
            }
            None => {return None}
        }
    }
}