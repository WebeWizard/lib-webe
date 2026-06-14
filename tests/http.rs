//! Facade-level smoke test: drive the web server through the `webe::web` re-export
//! exactly as a downstream user would, confirming the public surface is wired up.

use std::net::Ipv4Addr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use webe::web::responders::static_message::StaticResponder;
use webe::web::server::{Route, RouteMap, Server};

/// Starts a server on an OS-assigned port through the facade and returns its
/// bound address. The server runs on a spawned task for the test's duration.
async fn spawn_server() -> std::net::SocketAddr {
    let ip = Ipv4Addr::new(127, 0, 0, 1);
    let server = Server::new(&ip, &0)
        .await
        .expect("server should bind to an ephemeral port");
    let addr = server
        .local_addr()
        .expect("server should report its address");

    let mut routes = RouteMap::new();
    routes.add_route(
        Route::new("GET", "/hello"),
        StaticResponder::new(200, "hello from webe".to_owned()),
    );

    tokio::spawn(async move {
        let _ = server.start(routes).await;
    });

    addr
}

#[tokio::test]
async fn facade_serves_a_registered_route() {
    let addr = spawn_server().await;

    let mut stream = TcpStream::connect(addr)
        .await
        .expect("client should connect");
    stream
        .write_all(b"GET /hello HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .expect("client should send the request");
    stream.flush().await.expect("flush should succeed");

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .await
        .expect("client should read the response");

    assert!(
        response.starts_with("HTTP/1.1 200"),
        "expected a 200 status line, got: {response:?}"
    );
    assert!(
        response.contains("hello from webe"),
        "expected the responder body, got: {response:?}"
    );
}

#[tokio::test]
async fn facade_returns_404_for_unknown_path() {
    let addr = spawn_server().await;

    let mut stream = TcpStream::connect(addr)
        .await
        .expect("client should connect");
    stream
        .write_all(b"GET /missing HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .expect("client should send the request");
    stream.flush().await.expect("flush should succeed");

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .await
        .expect("client should read the response");

    assert!(
        response.starts_with("HTTP/1.1 404"),
        "expected a 404 status line, got: {response:?}"
    );
}
