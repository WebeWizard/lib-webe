use std::collections::HashMap;
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::io::{BufReader, BufWriter};
use std::time::Duration;
use std::sync::Arc;
use std::thread;

use super::request::Request;
use super::response::Response;
use super::responders::{Responder};
use super::responders::static_message::StaticResponder;
use super::status::Status;

pub struct Server {
    pub ip: Ipv4Addr,
    pub port: u16,
    listener: TcpListener,
    pub routes: Arc<HashMap<(String, String), Box<Responder + 'static>>>
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
                    routes: Arc::new(HashMap::<(String, String), Box<Responder>>::new())
                })
            },
            Err(error) => {return Err(ServerError::BindError(error))}
        };
    }

    pub fn add_route<T: Responder+'static>(&mut self, route: (String, String), responder: T) {
        match Arc::get_mut(&mut self.routes) { // TODO: not sure why this is necessary when already borrowing mut self
            Some(routes) => {
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
                        Server::process_stream(stream, &routes)
                    });
                },
                Err(error) => return Err(ServerError::ConnectionFailed(error))
            }
        }
        return Ok(());
    }

    // process a client request and give a result
    // TODO: handle these errors better (need to know the real error for logging, whatever)
    pub fn process_stream(stream: TcpStream, routes: &Arc<HashMap<(String,String),Box<Responder>>>) -> Result<(), ServerError> {
        // keep alive by default with 5 second timeout
        match stream.set_read_timeout(Some(Duration::from_secs(5))) {
            Ok(_) => {
                println!("set_read_timeout");
                let keep_alive = true;
                // get the request from the stream
                let buf_reader = BufReader::new(&stream);
                match Request::new(buf_reader) {
                    Ok(request) => {
                        println!("Request URI: {:?}",request.uri);
                        // TODO: determine if request needs decoded?
                        
                        // ~~ find the best responder ~~
                        // non-terminal route params WILL NOT contain more than one request uri part
                        // terminal route params WILL contain the remainder of the request uri
                        let request_uri_parts: Vec<&str> = request.uri.split('/').collect();
                        let request_uri_length = request_uri_parts.len();
                        let keys: Vec<&(String,String)> = routes.keys().collect();
                        match keys.iter().max_by_key(|key| {
                            // compare method
                            if key.0 != request.method {return 0}
                            let route_uri_parts: Vec<&str> = key.1.split('/').collect();
                            // compare length
                            if route_uri_parts.len() >= request_uri_length {return 0}
                            // find the one with the most matching parts
                            let mut size = 0;
                            for part in &request_uri_parts {
                                if part == &route_uri_parts[size] || route_uri_parts[size].contains('<') {
                                    size += 1;
                                } else { // uri doesn't match
                                    size = 0; 
                                    break;
                                }
                            }
                            return size;
                        }) {
                            Some(key) => {
                                match routes.get(key) {
                                    Some(responder) => {
                                        let mut params = HashMap::<String,String>::new();
                                        let route_uri_parts: Vec<&str> = key.1.split('/').collect();
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
                                                println!("Processed part: {} , {}",name,value);
                                                params.insert(name, value);
                                            }
                                        }
                                        
                                        match responder.validate(&request, &params) {
                                            Ok(validation_code) => {
                                                match responder.build_response(&request, &params, validation_code) {
                                                    Ok(mut response) => {
                                                        match response.respond(BufWriter::new(&stream)) {
                                                            Ok(()) => {},
                                                            Err(_error) => return Err(ServerError::InternalError)
                                                        }
                                                    },
                                                    Err(response_code) => {
                                                        match Status::from_code(response_code) {
                                                            Some(status) => {
                                                                let static_responder = StaticResponder::from_status(status);
                                                                match static_responder.build_response(&request, &params, response_code) {
                                                                    Ok(mut response) => {
                                                                        match response.respond(BufWriter::new(&stream)) {
                                                                            Ok(()) => {},
                                                                            Err(_error) => return Err(ServerError::InternalError)
                                                                        }
                                                                    },
                                                                    Err(_error) => return Err(ServerError::InternalError)
                                                                }
                                                            },
                                                            None => {
                                                                let static_responder = StaticResponder::new(response_code, String::new());
                                                                match static_responder.build_response(&request, &params, response_code) {
                                                                    Ok(mut response) => {
                                                                        match response.respond(BufWriter::new(&stream)) {
                                                                            Ok(()) => {},
                                                                            Err(_error) => return Err(ServerError::InternalError)
                                                                        }
                                                                    },
                                                                    Err(_error) => return Err(ServerError::InternalError)
                                                                }
                                                            }
                                                        }
                                                        
                                                    }
                                                }
                                            },
                                            Err(validation_code) => {
                                                match Status::from_code(validation_code) {
                                                    Some(status) => {
                                                        let static_responder = StaticResponder::from_status(status);
                                                        match static_responder.build_response(&request, &params, validation_code) {
                                                            Ok(mut response) => {
                                                                match response.respond(BufWriter::new(&stream)) {
                                                                    Ok(()) => {},
                                                                    Err(_error) => return Err(ServerError::InternalError)
                                                                }
                                                            },
                                                            Err(_error) => return Err(ServerError::InternalError)
                                                        }
                                                    },
                                                    None => {
                                                        let static_responder = StaticResponder::new(validation_code, String::new());
                                                        match static_responder.build_response(&request, &params, validation_code) {
                                                            Ok(mut response) => {
                                                                match response.respond(BufWriter::new(&stream)) {
                                                                    Ok(()) => {},
                                                                    Err(_error) => return Err(ServerError::InternalError)
                                                                }
                                                            },
                                                            Err(_error) => return Err(ServerError::InternalError)
                                                        }
                                                    }
                                                }
                                            }
                                        }                                        
                                    }
                                    None => return Err(ServerError::InternalError)
                                }
                            },
                            None => {} //TODO: respond with bad request
                        }
                    },
                    Err(_error) => return Err(ServerError::InternalError)
                }
            },
            Err(_) => return Err(ServerError::InternalError)
        }
        return Ok(());
    }
}