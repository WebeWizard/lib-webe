use crossbeam_utils::thread;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Read};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration;

use super::encoding::chunked::ChunkedDecoder;
use super::request::Request;
use super::responders::static_message::StaticResponder;
use super::responders::Responder;
use super::status::Status;

#[derive(PartialEq, Eq, Hash)]
pub struct Route {
  pub method: String,
  pub uri: String,
}

impl Route {
  pub fn new(method: &str, uri: &str) -> Route {
    Route {
      method: method.to_owned(),
      uri: uri.to_owned(),
    }
  }
}

type RouteMap<'r> = HashMap<Route, Box<dyn Responder + 'r>>;

pub struct Server<'r> {
  pub ip: Ipv4Addr,
  pub port: u16,
  pub routes: Arc<RouteMap<'r>>, // server must not live longer than routes
  listener: TcpListener,
}

#[derive(Debug)]
pub enum ServerError {
  BadRequest,                       // Request is unable to be processed by the server
  BindError(std::io::Error),        // server failed to bind on ip and port
  ConnectionFailed(std::io::Error), // server failed to grab connection from listener
  InternalError,                    // failed to process the stream
}

impl<'r> Server<'r> {
  pub fn new(ip: &Ipv4Addr, port: &u16) -> Result<Server<'r>, ServerError> {
    // attempt to bind the server to the specified ip and port
    match TcpListener::bind((ip.clone(), port.clone())) {
      Ok(listener) => {
        return Ok(Server {
          ip: ip.clone(),
          port: port.clone(),
          listener: listener,
          routes: Arc::new(HashMap::<Route, Box<dyn Responder + 'r>>::new()),
        });
      }
      Err(error) => return Err(ServerError::BindError(error)),
    };
  }

  pub fn add_route<T: 'r + Responder>(&mut self, mut route: Route, responder: T) {
    match Arc::get_mut(&mut self.routes) {
      // TODO: not sure why this is necessary when already borrowing mut self
      Some(routes) => {
        // remove leading / if any
        if !route.uri.starts_with('/') {
          route.uri = "/".to_owned() + route.uri.as_str();
        }
        routes.insert(route, Box::new(responder));
      }
      None => {} //TODO: should this return some error if we can't get the mutable ref?
    }
  }

  // starts the server, blocks the thread while the server is running
  pub fn start(&self) -> Result<(), ServerError> {
    println!("starting the server");
    match thread::scope(|s| {
      for stream in self.listener.incoming() {
        match stream {
          Ok(stream) => {
            s.spawn(move |_| process_stream(&stream, &self.routes));
          }
          Err(error) => return Err(ServerError::ConnectionFailed(error)),
        }
      }
      return Ok(());
    }) {
      Ok(_) => return Ok(()),
      Err(_) => return Ok(()),
    }
  }
}

fn find_best_route<'a>(request: &Request, routes: &'a Arc<RouteMap>) -> Option<&'a Route> {
  // ~~ find the best responder ~~
  // non-terminal route params WILL NOT contain more than one request uri part
  // terminal route params WILL contain the remainder of the request uri
  let request_uri_parts: Vec<&str> = request.uri.split('/').collect();
  let request_uri_length = request_uri_parts.len();
  // only keys with matching method
  let keys: Vec<&Route> = routes
    .keys()
    .filter(|key| key.method == request.method)
    .collect();
  let mut matched = false;
  match keys.iter().max_by_key(|route| {
    let route_uri_parts: Vec<&str> = route.uri.split('/').collect();
    // compare length
    let route_length = route_uri_parts.len();
    if route_length > request_uri_length {
      return 0;
    }
    // find the one with the most matching parts
    let mut match_size = 0;
    for part in &request_uri_parts {
      if part == &route_uri_parts[match_size] || route_uri_parts[match_size].contains('<') {
        match_size += 1;
        if match_size == route_length {
          break;
        } // full match
      } else {
        // uri doesn't match
        match_size = 0;
        break;
      }
    }
    if match_size > 0 {
      matched = true
    } // use this to determine if no routes found
    return match_size;
  }) {
    Some(key) => {
      if !matched {
        return None;
      };
      return Some(key);
    }
    None => return None,
  }
}

// process a client request, return a bool to keep connection alive
// TODO: handle these errors better (need to know the real error for logging, whatever)
fn process_stream<'s>(stream: &'s TcpStream, routes: &Arc<RouteMap>) -> Result<(), ServerError> {
  match stream.set_read_timeout(Some(Duration::from_secs(10))) {
    // TODO: set a write timeout before we send response
    Ok(_) => {
      let mut keep_alive = true;
      while keep_alive == true {
        // get the request from the stream
        let mut buf_reader = BufReader::new(stream);
        match Request::new(&mut buf_reader) {
          Ok(mut request) => {
            println!("Processing Request. URI: {:?}", request.uri);

            // use best route to respond to request
            match find_best_route(&request, routes) {
              Some(route) => {
                match routes.get(route) {
                  Some(responder) => {
                    // process any route parameters
                    let mut params = HashMap::<String, String>::new();
                    let request_uri_parts: Vec<&str> = request.uri.split('/').collect();
                    let route_uri_parts: Vec<&str> = route.uri.split('/').collect();
                    let part_length = route_uri_parts.len();
                    for i in 0..part_length {
                      // if this is the last part of the route
                      // and the part is a route param
                      // then grab the remaining parts from the request uri
                      if route_uri_parts[i].contains('<') {
                        let name = route_uri_parts[i].to_owned();
                        let value = if i == part_length - 1 {
                          request_uri_parts[i..].join("/")
                        } else {
                          request_uri_parts[i].to_owned()
                        };
                        params.insert(name, value);
                      }
                    }

                    // TODO: move respnder.validate() here

                    // use a trait object because the exact reader type is unknown at compile time
                    let mut body_reader: Box<dyn BufRead + 's> = Box::new(buf_reader);

                    // using transfer encodings on the body?
                    match request.headers.get("transfer-encoding") {
                      Some(value) => {
                        let encodings: Vec<String> =
                          value.split(',').map(|e| e.trim().to_lowercase()).collect();
                        if encodings.len() >= 1 {
                          if encodings[encodings.len() - 1] != "chunked" {
                            // if not chunked, then assume connection will close
                            // unless content-length is given below
                            keep_alive = false;
                          }
                          // apply decoders in order
                          for encoding in encodings {
                            body_reader = match encoding.as_str() {
                              // TODO: Add gzip/deflate encoders/decoders
                              "chunked" => {
                                Box::new(BufReader::new(ChunkedDecoder::new(body_reader)))
                              }
                              "identity" => body_reader,
                              _ => return Err(ServerError::BadRequest),
                            }
                          }
                        }
                      }
                      None => {}
                    }

                    match request.headers.get("content-length") {
                      Some(value) => match value.parse::<u64>() {
                        Ok(content_length) => {
                          body_reader = Box::new(body_reader.take(content_length));
                          keep_alive = true
                        }
                        Err(_error) => return Err(ServerError::BadRequest),
                      },
                      None => {}
                    }

                    request.set_message_body(Some(body_reader));

                    // does request want to close connection?
                    match request.headers.get("connection") {
                      Some(con_header) => {
                        BufReader::new(stream);
                        if con_header.to_lowercase() == "close" {
                          keep_alive = false
                        }
                      }
                      None => {}
                    }

                    // TODO: move validate to before body reader is built
                    match responder.validate(&request, &params) {
                      Ok(validation_result) => {
                        match responder.build_response(&mut request, &params, validation_result) {
                          Ok(mut response) => match response.respond(BufWriter::new(&stream)) {
                            Ok(()) => keep_alive = response.keep_alive,
                            Err(_error) => return Err(ServerError::InternalError),
                          },
                          Err(response_code) => {
                            let static_responder =
                              StaticResponder::from_standard_code(response_code);
                            match static_responder.build_response(
                              &mut request,
                              &params,
                              None,
                            ) {
                              Ok(mut response) => {
                                match response.respond(BufWriter::new(&stream)) {
                                  Ok(()) => {} // keep_alive = true
                                  Err(_error) => return Err(ServerError::InternalError),
                                }
                              }
                              Err(_error) => return Err(ServerError::InternalError),
                            }
                          }
                        }
                      }
                      Err(validation_status) => {
                        let static_responder = StaticResponder::from_status(validation_status);
                        match static_responder.build_response(
                          &mut request,
                          &params,
                          None,
                        ) {
                          Ok(mut response) => {
                            match response.respond(BufWriter::new(&stream)) {
                              Ok(()) => {} // keep-alive = true
                              Err(_error) => return Err(ServerError::InternalError),
                            }
                          }
                          Err(_error) => return Err(ServerError::InternalError),
                        }
                      }
                    }
                  }
                  None => return Err(ServerError::InternalError),
                }
              }
              None => {
                let static_responder = StaticResponder::from_standard_code(400);
                match static_responder.build_response(
                  &mut request,
                  &HashMap::<String, String>::new(),
                  None,
                ) {
                  Ok(mut response) => {
                    match response.respond(BufWriter::new(&stream)) {
                      Ok(()) => {} //keep-alive = true
                      Err(_error) => return Err(ServerError::InternalError),
                    }
                  }
                  Err(_error) => return Err(ServerError::InternalError),
                }
              }
            }
          }
          Err(_error) => return Err(ServerError::InternalError),
        }
      }
    }
    Err(_) => return Err(ServerError::InternalError),
  }
  return Ok(());
}
