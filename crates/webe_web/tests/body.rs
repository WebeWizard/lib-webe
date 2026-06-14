//! Integration tests for request body framing (US3).

mod common;

use common::{EchoBodyResponder, TestClient, spawn_server};
use webe_web::server::{Route, RouteMap};

fn echo_routes() -> RouteMap<'static> {
    let mut map = RouteMap::new();
    map.add_route(Route::new("POST", "/echo"), EchoBodyResponder);
    map
}

#[tokio::test]
async fn content_length_body_is_read_exactly() {
    let addr = spawn_server(echo_routes()).await;
    let response = TestClient::request(
        addr,
        b"POST /echo HTTP/1.1\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhello",
    )
    .await;
    assert_eq!(response.status, 200);
    assert_eq!(response.body_string(), "hello");
}

#[tokio::test]
async fn chunked_body_is_decoded() {
    let addr = spawn_server(echo_routes()).await;
    let response = TestClient::request(
        addr,
        b"POST /echo HTTP/1.1\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n5\r\nhello\r\n0\r\n\r\n",
    )
    .await;
    assert_eq!(response.status, 200);
    assert_eq!(response.body_string(), "hello");
}

#[tokio::test]
async fn both_framing_headers_present_is_bad_request() {
    let addr = spawn_server(echo_routes()).await;
    let response = TestClient::request(
        addr,
        b"POST /echo HTTP/1.1\r\nContent-Length: 5\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\nhello",
    )
    .await;
    assert_eq!(response.status, 400);
    // the responder must not have run (its echo would have produced "hello")
    assert_ne!(response.body_string(), "hello");
}

#[tokio::test]
async fn unparseable_content_length_is_bad_request() {
    let addr = spawn_server(echo_routes()).await;
    let response = TestClient::request(
        addr,
        b"POST /echo HTTP/1.1\r\nContent-Length: not-a-number\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_eq!(response.status, 400);
}

#[tokio::test]
async fn unsupported_transfer_coding_is_rejected() {
    let addr = spawn_server(echo_routes()).await;
    let response = TestClient::request(
        addr,
        b"POST /echo HTTP/1.1\r\nTransfer-Encoding: gzip\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_eq!(response.status, 400);
    assert_ne!(response.body_string(), "hello");
}
