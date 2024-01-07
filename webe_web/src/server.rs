use std::cmp::Ordering::*;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::Arc;

use tokio::io::{AsyncBufRead, AsyncReadExt, BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream};

use super::encoding::chunked::ChunkedDecoder;
use super::request::{Request, RequestError};
use super::responders::static_message::StaticResponder;
use super::responders::Responder;
use super::response::ResponseError;

#[derive(PartialEq, Eq, Hash)]
pub struct Route {
    pub method: String,
    pub uri: String,
    pub has_params: bool,
}

impl Route {
    pub fn new(method: &str, uri: &str) -> Route {
        Route {
            method: method.to_owned(),
            uri: uri.to_owned(),
            has_params: uri.contains('<'),
        }
    }
}

pub struct RouteMap<'r> {
    inner: HashMap<Route, Box<dyn Responder + 'r>>,
}

impl<'r> RouteMap<'r> {
    pub fn new() -> RouteMap<'r> {
        RouteMap {
            inner: HashMap::new(),
        }
    }

    pub fn add_route<T: 'r + Responder>(&mut self, mut route: Route, responder: T) {
        // remove leading / if any
        if !route.uri.starts_with('/') {
            route.uri = "/".to_owned() + route.uri.as_str();
        }
        self.inner.insert(route, Box::new(responder));
    }
}

pub struct Server {
    pub ip: Ipv4Addr,
    pub port: u16,
    listener: TcpListener,
}

#[derive(Debug)]
pub enum ServerError {
    BadRequest(RequestError),  // Request is unable to be processed by the server
    BindError(std::io::Error), // server failed to bind on ip and port
    ConnectionFailed(std::io::Error), // server failed to grab connection from listener
    InternalError,             // failed to process the stream
}

impl From<RequestError> for ServerError {
    fn from(err: RequestError) -> ServerError {
        ServerError::BadRequest(err)
    }
}

impl From<ResponseError> for ServerError {
    fn from(_err: ResponseError) -> ServerError {
        ServerError::InternalError
    }
}

impl Server {
    pub async fn new(ip: &Ipv4Addr, port: &u16) -> Result<Server, ServerError> {
        // attempt to bind the server to the specified ip and port
        match TcpListener::bind((ip.clone(), port.clone())).await {
            Ok(listener) => {
                return Ok(Server {
                    ip: ip.clone(),
                    port: port.clone(),
                    listener: listener,
                });
            }
            Err(error) => return Err(ServerError::BindError(error)),
        };
    }

    // starts the server, blocks the thread while the server is running
    pub async fn start(&self, routes: RouteMap<'static>) -> Result<(), ServerError> {
        let routes_arc = Arc::new(routes);
        loop {
            match self.listener.accept().await {
                Ok((stream, _socket)) => {
                    let process_routes = routes_arc.clone();
                    tokio::spawn(async move { process_stream(stream, process_routes).await });
                }
                Err(error) => return Err(ServerError::ConnectionFailed(error)),
            }
        }
    }
}

fn find_best_route<'r>(request: &Request, routes: &'r Arc<RouteMap<'r>>) -> Option<&'r Route> {
    // ~~ find the best responder ~~
    // first check for an exact match
    if let Some(route) = routes.inner.keys().find(|route| {
        !route.has_params && request.method == route.method && route.uri == request.uri
    }) {
        return Some(route);
    }
    // non-terminal route params WILL NOT contain more than one request uri part
    // terminal route params WILL contain the remainder of the request uri
    let request_parts: Vec<&str> = request.uri.split('/').collect();
    // only keys with matching method
    match routes
        .inner
        .keys()
        .filter_map(|route| {
            if route.method != request.method {
                return None;
            }

            let route_parts: Vec<&str> = route.uri.split('/').collect();
            // compare length. route cannot match request with less parts
            if route_parts.len() > request_parts.len() {
                return None;
            }
            // find the one with the most matching parts
            let mut match_size = 0;
            let mut first_wild = 0;
            for i in 0..request_parts.len() {
                if request_parts[i] == route_parts[i] || route_parts[i].contains('<') {
                    match_size = i + 1;
                    if first_wild == 0 && route_parts[i].contains('<') {
                        first_wild = i + 1;
                    }
                    if (i + 1) == route_parts.len() {
                        break;
                    }
                } else {
                    return None;
                } // uri doesn't match
            }
            return Some((route, match_size, first_wild));
        })
        .max_by(|x, y| match (x.1).cmp(&y.1) {
            Less => return Less,
            Greater => return Greater,
            Equal => ((x.2).cmp(&y.2)).reverse(),
        }) {
        Some((route, _, _)) => return Some(route),
        None => return None,
    }
}

fn parse_route_params(request: &Request, route: &Route) -> Vec<(String, String)> {
    // I can't imagine a request having too many params,
    // so a Vec should be generally much faster than hashmap of small size
    let mut params: Vec<(String, String)> = Vec::new();

    if !route.has_params {
        return params;
    }

    let request_parts: Vec<&str> = request.uri.split('/').collect();
    let route_uri_parts: Vec<&str> = route.uri.split('/').collect();
    let part_length = route_uri_parts.len();
    for i in 0..part_length {
        if route_uri_parts[i].contains('<') {
            let name = route_uri_parts[i].to_owned();
            let value = if i == part_length - 1 {
                // if this is the last part of the route and the part is a route param...
                // then combine the remaining parts from the request uri (ex. a path to a subfolder)
                request_parts[i..].join("/")
            } else {
                request_parts[i].to_owned()
            };
            params.push((name, value));
        }
    }
    return params;
}

// process a client request
// TODO: handle these errors better (need to know the real error for logging, whatever)
async fn process_stream(
    mut stream: TcpStream,
    routes: Arc<RouteMap<'_>>,
) -> Result<(), ServerError> {
    // split the stream into reader and writer
    let (reader, writer) = stream.split();
    let mut buf_reader = BufReader::new(reader);
    let mut buf_writer = BufWriter::new(writer);

    let mut keep_alive = true; // keep-alive by default
    while keep_alive {
        let responder; // placeholder for selected responder
        let mut response; // placeholder for some kind of response
        match Request::new(&mut buf_reader).await {
            Ok(mut request) => {
                // find a route for the request , or return 404 Not Found
                if let Some(route) = find_best_route(&request, &routes) {
                    responder = routes.inner.get(route).unwrap(); // safe to unwrap here because because we know route exists
                    let params = parse_route_params(&request, route);
                    match request.parse_headers(&mut buf_reader).await {
                        Ok(()) => {
                            // use a trait object because the final reader type is unknown at compile time
                            let mut body_reader: std::pin::Pin<
                                Box<dyn AsyncBufRead + Send + Sync>,
                            > = Box::pin(&mut buf_reader);

                            // using transfer encodings on the body?
                            match &request.headers {
                                Some(req_headers) => {
                                    match req_headers.get("transfer-encoding") {
                                        Some(value) => {
                                            let encodings: Vec<String> = value
                                                .split(',')
                                                .map(|e| e.trim().to_lowercase())
                                                .collect();
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
                                                        "chunked" => Box::pin(BufReader::new(
                                                            ChunkedDecoder::new(body_reader),
                                                        )),
                                                        "identity" => body_reader,
                                                        _ => return Err(ServerError::BadRequest(
                                                            RequestError::EncodingNotSupportedError,
                                                        )),
                                                    }
                                                }
                                            }
                                        }
                                        None => {}
                                    }

                                    match req_headers.get("content-length") {
                                        Some(value) => match value.parse::<u64>() {
                                            Ok(content_length) => {
                                                body_reader =
                                                    Box::pin(body_reader.take(content_length));
                                                keep_alive = true
                                            }
                                            Err(_error) => {
                                                return Err(ServerError::BadRequest(
                                                    RequestError::MalformedRequestError,
                                                ))
                                            }
                                        },
                                        None => {}
                                    }
                                    // does request want to close connection?
                                    match req_headers.get("connection") {
                                        Some(con_header) => {
                                            if con_header.to_lowercase() == "close" {
                                                keep_alive = false
                                            }
                                        }
                                        None => {}
                                    }
                                }
                                None => {}
                            }

                            request.set_message_body(Some(body_reader));

                            // validate the request is able to be responded to with the selected responder
                            if let Ok(validation_result) =
                                responder.validate(&request, &params, None)
                            {
                                match responder
                                    .build_response(&mut request, &params, validation_result)
                                    .await
                                {
                                    Ok(new_response) => response = new_response,
                                    Err(error_code) => {
                                        response = StaticResponder::from_standard_code(error_code)
                                            .quick_response()
                                    }
                                }
                            } else {
                                response = StaticResponder::from_standard_code(400).quick_response()
                            } // 400 Bad Request
                        }
                        Err(_error) => {
                            response = StaticResponder::from_standard_code(400).quick_response()
                        }
                    }
                } else {
                    response = StaticResponder::from_standard_code(404).quick_response();
                }
            }
            Err(_error) => response = StaticResponder::from_standard_code(400).quick_response(), // 400 Bad Request
        }

        response.respond(&mut buf_writer).await? // TODO: do we need to await here? or just let it fly?
    }

    return Ok(());
}
