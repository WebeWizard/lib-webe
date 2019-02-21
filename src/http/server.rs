use std::net::{IpAddr, TcpListener, TcpStream};
use std::io::{BufReader, BufWriter};
use std::time::Duration;
use std::thread;

use super::request::Request;
use super::response::Response;

pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
    listener: TcpListener
}

pub enum ServerError {
    BindError(std::io::Error), // server failed to bind on ip and port
    ConnectionFailed(std::io::Error), // server failed to grab connection from listener
    InternalError, // failed to process the stream
}

impl Server {
    pub fn new(ip: &IpAddr, port: &u16) -> Result<Server, ServerError> {
        // attempt to bind the server to the specified ip and port
        match TcpListener::bind((ip.clone(), port.clone())) {
            Ok(listener) => {
                return Ok(Server{ 
                    ip: ip.clone(),
                    port: port.clone(),
                    listener: listener
                })
            },
            Err(error) => {return Err(ServerError::BindError(error))}
        };
    }

    // starts the server, blocks the thread while the server is running
    pub fn start(&self) -> Result<(), ServerError> {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        Server::process_stream(stream)
                    });
                },
                Err(error) => return Err(ServerError::ConnectionFailed(error))
            }
        }
        return Ok(());
    }

    // process a client request and give a result
    // TODO: handle these errors better (need to know the real error for logging, whatever)
    pub fn process_stream(stream: TcpStream) -> Result<(), ServerError> {
        // keep alive by default with 5 second timeout
        match stream.set_read_timeout(Some(Duration::from_secs(5))) {
            Ok(_) => {
                let keep_alive = true;
                // get the request from the stream
                let buf_reader = BufReader::new(&stream);
                let request = Request::new(buf_reader);
                // TODO: determine if request needs decoded?
                // prepare the response
                match Response::new(&request) { // 
                    Ok(response) => {
                        // send the response
                        let buf_writer = BufWriter::new(&stream);
                        match response.respond(buf_writer) {
                            Ok(_) => return Ok(()),
                            Err(error) => return Err(ServerError::InternalError)
                        }
                    },
                    Err(error) => return Err(ServerError::InternalError)
                } 
            },
            Err(_) => return Err(ServerError::InternalError)
        }
        return Ok(());
    }
}