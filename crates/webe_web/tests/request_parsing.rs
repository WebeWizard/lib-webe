//! Integration tests for request parsing and version enforcement (US1).

mod common;

use std::net::Ipv4Addr;

use common::{LabelResponder, TestClient, spawn_server};
use webe_web::error::WebError;
use webe_web::server::{Route, RouteMap, Server};

fn routes() -> RouteMap<'static> {
    let mut map = RouteMap::new();
    map.add_route(Route::new("GET", "/"), LabelResponder::new("root"));
    map
}

#[tokio::test]
async fn valid_http_1_1_request_is_accepted() {
    let addr = spawn_server(routes()).await;
    let response = TestClient::request(
        addr,
        b"GET / HTTP/1.1\r\nHost: test\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_eq!(response.status, 200);
    assert_eq!(response.body_string(), "root");
}

#[tokio::test]
async fn malformed_request_line_is_bad_request() {
    let addr = spawn_server(routes()).await;
    // only two parts instead of METHOD SP target SP version
    let response = TestClient::request(addr, b"GET /\r\nConnection: close\r\n\r\n").await;
    assert_eq!(response.status, 400);
}

#[tokio::test]
async fn missing_header_separator_is_bad_request() {
    let addr = spawn_server(routes()).await;
    let response =
        TestClient::request(addr, b"GET / HTTP/1.1\r\nthis-header-has-no-colon\r\n\r\n").await;
    assert_eq!(response.status, 400);
}

#[tokio::test]
async fn request_line_that_never_terminates_is_bad_request() {
    // A request line that exceeds the limit (or never terminates) yields the
    // same MaxURISizeError -> 400 outcome. We use a half-closed connection with
    // no terminating CRLF to exercise it deterministically; the true over-limit
    // path is covered by the unit tests in `src/request.rs`.
    let addr = spawn_server(routes()).await;
    let mut client = TestClient::connect(addr).await;
    client
        .send(b"GET /a-request-line-with-no-terminator HTTP/1.1")
        .await;
    client.shutdown_write().await;
    let response = client.recv().await;
    assert_eq!(response.status, 400);
}

#[tokio::test]
async fn non_http_1_1_version_is_unsupported() {
    let addr = spawn_server(routes()).await;
    let response = TestClient::request(addr, b"GET / HTTP/1.0\r\nConnection: close\r\n\r\n").await;
    assert_eq!(response.status, 505);
}

#[tokio::test]
async fn binding_an_already_bound_port_is_a_typed_bind_error() {
    let ip = Ipv4Addr::new(127, 0, 0, 1);
    let first = Server::new(&ip, &0)
        .await
        .expect("first bind should succeed");
    let port = first.local_addr().expect("local addr").port();

    let result = Server::new(&ip, &port).await;
    assert!(
        matches!(result, Err(WebError::Bind(_))),
        "re-binding a live port should return WebError::Bind, got {:?}",
        result.err()
    );
}
