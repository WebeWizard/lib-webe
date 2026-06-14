//! Integration tests for response framing and keep-alive vs close (US3).

mod common;

use async_trait::async_trait;

use common::{LabelResponder, StreamResponder, TestClient, spawn_server};
use webe_web::request::Request;
use webe_web::responders::Responder;
use webe_web::response::Response;
use webe_web::server::{Route, RouteMap};
use webe_web::validation::Validation;

/// A responder that returns a bodyless `204` response.
struct EmptyResponder;

#[async_trait]
impl Responder for EmptyResponder {
    async fn build_response(
        &self,
        _request: &mut Request,
        _params: &Vec<(String, String)>,
        _validation: Validation,
    ) -> Result<Response, u16> {
        Ok(Response::new(204))
    }
}

fn routes() -> RouteMap<'static> {
    let mut map = RouteMap::new();
    map.add_route(
        Route::new("GET", "/known"),
        LabelResponder::new("known-body"),
    );
    map.add_route(
        Route::new("GET", "/stream"),
        StreamResponder {
            body: b"streamed payload".to_vec(),
        },
    );
    map.add_route(Route::new("GET", "/empty"), EmptyResponder);
    map
}

#[tokio::test]
async fn known_length_body_sends_content_length() {
    let addr = spawn_server(routes()).await;
    let response =
        TestClient::request(addr, b"GET /known HTTP/1.1\r\nConnection: close\r\n\r\n").await;
    assert_eq!(response.status, 200);
    assert!(response.header("Content-Length").is_some());
    assert!(response.header("Transfer-Encoding").is_none());
    assert_eq!(response.body_string(), "known-body");
}

#[tokio::test]
async fn streamed_unknown_length_body_sends_chunked() {
    let addr = spawn_server(routes()).await;
    let response =
        TestClient::request(addr, b"GET /stream HTTP/1.1\r\nConnection: close\r\n\r\n").await;
    assert_eq!(response.status, 200);
    assert_eq!(
        response
            .header("Transfer-Encoding")
            .map(|v| v.to_lowercase()),
        Some("chunked".to_string())
    );
    assert!(response.header("Content-Length").is_none());
    assert_eq!(response.body_string(), "streamed payload");
}

#[tokio::test]
async fn bodyless_response_sends_neither_framing_header() {
    let addr = spawn_server(routes()).await;
    let response =
        TestClient::request(addr, b"GET /empty HTTP/1.1\r\nConnection: close\r\n\r\n").await;
    assert_eq!(response.status, 204);
    assert!(response.header("Content-Length").is_none());
    assert!(response.header("Transfer-Encoding").is_none());
    assert!(response.body.is_empty());
}

#[tokio::test]
async fn two_keep_alive_requests_on_one_connection() {
    let addr = spawn_server(routes()).await;
    let mut client = TestClient::connect(addr).await;

    // first request keeps the connection alive
    client.send(b"GET /known HTTP/1.1\r\n\r\n").await;
    let first = client.recv().await;
    assert_eq!(first.status, 200);
    assert_eq!(first.body_string(), "known-body");

    // second request on the SAME connection, closing afterward
    client
        .send(b"GET /known HTTP/1.1\r\nConnection: close\r\n\r\n")
        .await;
    let second = client.recv().await;
    assert_eq!(second.status, 200);
    assert_eq!(second.body_string(), "known-body");
}

#[tokio::test]
async fn connection_close_is_honored() {
    let addr = spawn_server(routes()).await;
    let response =
        TestClient::request(addr, b"GET /known HTTP/1.1\r\nConnection: close\r\n\r\n").await;
    assert_eq!(response.status, 200);
    assert_eq!(
        response.header("Connection").map(|v| v.to_lowercase()),
        Some("close".to_string())
    );
}
