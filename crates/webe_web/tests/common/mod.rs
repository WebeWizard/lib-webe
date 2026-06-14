//! Shared helpers for the `webe_web` integration tests.
//!
//! Spawns a real server on an OS-assigned port and provides a tiny HTTP client
//! that can pipeline requests and parse framed responses (`Content-Length` and
//! `chunked`), which the keep-alive tests rely on.

#![allow(dead_code)]

use std::collections::HashMap;
use std::io::Cursor;
use std::net::{Ipv4Addr, SocketAddr};

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use webe_web::request::Request;
use webe_web::responders::Responder;
use webe_web::response::Response;
use webe_web::server::{RouteMap, Server};
use webe_web::validation::Validation;

/// Binds a server on `127.0.0.1:0`, starts it on a background task, and returns
/// the OS-assigned address to connect to.
pub async fn spawn_server(routes: RouteMap<'static>) -> SocketAddr {
    let ip = Ipv4Addr::new(127, 0, 0, 1);
    let port: u16 = 0;
    let server = Server::new(&ip, &port)
        .await
        .expect("server should bind on an ephemeral port");
    let addr = server
        .local_addr()
        .expect("server should report its local address");
    tokio::spawn(async move {
        let _ = server.start(routes).await;
    });
    addr
}

/// A parsed HTTP response.
#[derive(Debug)]
pub struct TestResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl TestResponse {
    pub fn body_string(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers
            .keys()
            .find(|k| k.eq_ignore_ascii_case(name))
            .and_then(|k| self.headers.get(k))
    }
}

/// A minimal pipelining HTTP client over one TCP connection.
pub struct TestClient {
    stream: TcpStream,
}

impl TestClient {
    pub async fn connect(addr: SocketAddr) -> TestClient {
        let stream = TcpStream::connect(addr)
            .await
            .expect("client should connect");
        TestClient { stream }
    }

    /// Connects, sends one raw request, reads one response, and returns it.
    pub async fn request(addr: SocketAddr, raw: &[u8]) -> TestResponse {
        let mut client = TestClient::connect(addr).await;
        client.send(raw).await;
        client.recv().await
    }

    pub async fn send(&mut self, raw: &[u8]) {
        self.stream.write_all(raw).await.expect("client write");
        self.stream.flush().await.expect("client flush");
    }

    /// Half-closes the write side so the server observes EOF on the request.
    pub async fn shutdown_write(&mut self) {
        let _ = self.stream.shutdown().await;
    }

    pub async fn recv(&mut self) -> TestResponse {
        let status_line = self.read_line().await;
        let status = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|code| code.parse::<u16>().ok())
            .expect("response status line should contain a status code");

        let mut headers = HashMap::new();
        loop {
            let line = self.read_line().await;
            let trimmed = line.trim_end_matches(['\r', '\n']);
            if trimmed.is_empty() {
                break;
            }
            if let Some((name, value)) = trimmed.split_once(':') {
                headers.insert(name.trim().to_string(), value.trim().to_string());
            }
        }

        let body = self.read_body(&headers).await;
        TestResponse {
            status,
            headers,
            body,
        }
    }

    async fn read_body(&mut self, headers: &HashMap<String, String>) -> Vec<u8> {
        let lookup = |name: &str| {
            headers
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(name))
                .map(|(_, v)| v.clone())
        };

        if let Some(te) = lookup("transfer-encoding")
            && te.to_lowercase().contains("chunked")
        {
            return self.read_chunked_body().await;
        }
        if let Some(len) = lookup("content-length") {
            let n: usize = len.trim().parse().unwrap_or(0);
            let mut buf = vec![0u8; n];
            self.stream
                .read_exact(&mut buf)
                .await
                .expect("client should read the full content-length body");
            return buf;
        }
        Vec::new()
    }

    async fn read_chunked_body(&mut self) -> Vec<u8> {
        let mut body = Vec::new();
        loop {
            let size_line = self.read_line().await;
            let size = usize::from_str_radix(size_line.trim_end_matches(['\r', '\n']), 16)
                .expect("chunk size should be valid hex");
            if size == 0 {
                let _ = self.read_line().await; // trailing CRLF after the final chunk
                break;
            }
            let mut chunk = vec![0u8; size];
            self.stream
                .read_exact(&mut chunk)
                .await
                .expect("client should read a full chunk");
            body.extend_from_slice(&chunk);
            let _ = self.read_line().await; // CRLF after the chunk data
        }
        body
    }

    async fn read_line(&mut self) -> String {
        let mut line = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            match self.stream.read(&mut byte).await {
                Ok(0) => break,
                Ok(_) => {
                    line.push(byte[0]);
                    if byte[0] == b'\n' {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        String::from_utf8_lossy(&line).into_owned()
    }
}

/// A responder that returns `200` with a body of `label` plus every captured
/// parameter as `;name=value`, so tests can tell which route matched and what
/// parameters reached the responder.
pub struct LabelResponder {
    pub label: String,
}

impl LabelResponder {
    pub fn new(label: &str) -> LabelResponder {
        LabelResponder {
            label: label.to_string(),
        }
    }
}

#[async_trait]
impl Responder for LabelResponder {
    async fn build_response(
        &self,
        _request: &mut Request,
        params: &Vec<(String, String)>,
        _validation: Validation,
    ) -> Result<Response, u16> {
        let mut body = self.label.clone();
        for (name, value) in params {
            body.push_str(&format!(";{name}={value}"));
        }
        let bytes = body.into_bytes();
        let mut response = Response::new(200);
        response
            .headers
            .insert("Content-Length".to_owned(), bytes.len().to_string());
        response.message_body = Some(Box::pin(Cursor::new(bytes)));
        Ok(response)
    }
}

/// A responder that returns `200` with a body but **no** `Content-Length`, which
/// forces the server to frame the response with `Transfer-Encoding: chunked`.
pub struct StreamResponder {
    pub body: Vec<u8>,
}

#[async_trait]
impl Responder for StreamResponder {
    async fn build_response(
        &self,
        _request: &mut Request,
        _params: &Vec<(String, String)>,
        _validation: Validation,
    ) -> Result<Response, u16> {
        let mut response = Response::new(200);
        // intentionally no Content-Length -> chunked framing
        response.message_body = Some(Box::pin(Cursor::new(self.body.clone())));
        Ok(response)
    }
}

/// A responder that reads the request body to end and echoes it back with a
/// `Content-Length`, so tests can verify the body was framed and delivered.
pub struct EchoBodyResponder;

#[async_trait]
impl Responder for EchoBodyResponder {
    async fn build_response(
        &self,
        request: &mut Request,
        _params: &Vec<(String, String)>,
        _validation: Validation,
    ) -> Result<Response, u16> {
        let mut buf = Vec::new();
        if let Some(body) = request.message_body.as_mut()
            && body.read_to_end(&mut buf).await.is_err()
        {
            return Err(500);
        }
        let mut response = Response::new(200);
        response
            .headers
            .insert("Content-Length".to_owned(), buf.len().to_string());
        response.message_body = Some(Box::pin(Cursor::new(buf)));
        Ok(response)
    }
}
