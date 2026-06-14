//! Integration tests for routing: 404/405 distinction (US1) and deterministic
//! selection + parameter capture (US2).

mod common;

use common::{LabelResponder, TestClient, spawn_server};
use webe_web::server::{Route, RouteMap};

// ---------- User Story 1: reliable 404 / 405 ----------

#[tokio::test]
async fn registered_route_returns_responder_output() {
    let mut map = RouteMap::new();
    map.add_route(Route::new("GET", "/hello"), LabelResponder::new("hello"));
    let addr = spawn_server(map).await;

    let response =
        TestClient::request(addr, b"GET /hello HTTP/1.1\r\nConnection: close\r\n\r\n").await;
    assert_eq!(response.status, 200);
    assert_eq!(response.body_string(), "hello");
}

#[tokio::test]
async fn unregistered_path_is_not_found() {
    let mut map = RouteMap::new();
    map.add_route(Route::new("GET", "/hello"), LabelResponder::new("hello"));
    let addr = spawn_server(map).await;

    let response =
        TestClient::request(addr, b"GET /missing HTTP/1.1\r\nConnection: close\r\n\r\n").await;
    assert_eq!(response.status, 404);
}

#[tokio::test]
async fn path_match_with_wrong_method_is_method_not_allowed() {
    let mut map = RouteMap::new();
    map.add_route(Route::new("GET", "/widget"), LabelResponder::new("widget"));
    let addr = spawn_server(map).await;

    let response =
        TestClient::request(addr, b"POST /widget HTTP/1.1\r\nConnection: close\r\n\r\n").await;
    assert_eq!(response.status, 405);
}

#[tokio::test]
async fn server_remains_available_after_each_failure() {
    let mut map = RouteMap::new();
    map.add_route(Route::new("GET", "/ok"), LabelResponder::new("ok"));
    let addr = spawn_server(map).await;

    // a 404, a 405, then a successful request — the server must stay up
    let not_found =
        TestClient::request(addr, b"GET /nope HTTP/1.1\r\nConnection: close\r\n\r\n").await;
    assert_eq!(not_found.status, 404);

    let wrong_method =
        TestClient::request(addr, b"DELETE /ok HTTP/1.1\r\nConnection: close\r\n\r\n").await;
    assert_eq!(wrong_method.status, 405);

    let ok = TestClient::request(addr, b"GET /ok HTTP/1.1\r\nConnection: close\r\n\r\n").await;
    assert_eq!(ok.status, 200);
    assert_eq!(ok.body_string(), "ok");
}

// ---------- User Story 2: deterministic selection + params ----------

#[tokio::test]
async fn exact_route_wins_over_parameterized_route() {
    let mut map = RouteMap::new();
    map.add_route(
        Route::new("GET", "/files/list"),
        LabelResponder::new("exact"),
    );
    map.add_route(
        Route::new("GET", "/files/<name>"),
        LabelResponder::new("param"),
    );
    let addr = spawn_server(map).await;

    let exact = TestClient::request(
        addr,
        b"GET /files/list HTTP/1.1\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_eq!(exact.body_string(), "exact");

    let param = TestClient::request(
        addr,
        b"GET /files/readme HTTP/1.1\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_eq!(param.body_string(), "param;<name>=readme");
}

#[tokio::test]
async fn earliest_wildcard_breaks_ties() {
    // Both routes match `/a/b/c` with three matching parts; the one whose
    // wildcard appears earliest is selected.
    let mut map = RouteMap::new();
    map.add_route(Route::new("GET", "/<x>/b/c"), LabelResponder::new("early"));
    map.add_route(Route::new("GET", "/a/<y>/c"), LabelResponder::new("late"));
    let addr = spawn_server(map).await;

    let response =
        TestClient::request(addr, b"GET /a/b/c HTTP/1.1\r\nConnection: close\r\n\r\n").await;
    assert_eq!(response.body_string(), "early;<x>=a");
}

#[tokio::test]
async fn leading_slash_and_no_leading_slash_match_the_same_path() {
    let mut map = RouteMap::new();
    // registered without a leading slash; must still match `/things`
    map.add_route(Route::new("GET", "things"), LabelResponder::new("things"));
    let addr = spawn_server(map).await;

    let response =
        TestClient::request(addr, b"GET /things HTTP/1.1\r\nConnection: close\r\n\r\n").await;
    assert_eq!(response.status, 200);
    assert_eq!(response.body_string(), "things");
}

#[tokio::test]
async fn non_terminal_param_captures_one_segment() {
    let mut map = RouteMap::new();
    map.add_route(
        Route::new("GET", "/user/<id>/profile"),
        LabelResponder::new("profile"),
    );
    let addr = spawn_server(map).await;

    let response = TestClient::request(
        addr,
        b"GET /user/42/profile HTTP/1.1\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_eq!(response.body_string(), "profile;<id>=42");
}

#[tokio::test]
async fn terminal_param_captures_remaining_path() {
    let mut map = RouteMap::new();
    map.add_route(
        Route::new("GET", "/assets/<path>"),
        LabelResponder::new("asset"),
    );
    let addr = spawn_server(map).await;

    let response = TestClient::request(
        addr,
        b"GET /assets/css/site/main.css HTTP/1.1\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_eq!(response.body_string(), "asset;<path>=css/site/main.css");
}

#[tokio::test]
async fn selection_is_stable_across_many_routes() {
    let mut map = RouteMap::new();
    // a mix of exact and parameterized routes (>= 25)
    for i in 0..20 {
        map.add_route(
            Route::new("GET", &format!("/exact/{i}")),
            LabelResponder::new(&format!("exact-{i}")),
        );
    }
    for i in 0..10 {
        map.add_route(
            Route::new("GET", &format!("/param/{i}/<rest>")),
            LabelResponder::new(&format!("param-{i}")),
        );
    }
    let addr = spawn_server(map).await;

    // repeated identical requests must always select the same responder
    for _ in 0..5 {
        let exact =
            TestClient::request(addr, b"GET /exact/7 HTTP/1.1\r\nConnection: close\r\n\r\n").await;
        assert_eq!(exact.body_string(), "exact-7");

        let param = TestClient::request(
            addr,
            b"GET /param/3/deep/leaf HTTP/1.1\r\nConnection: close\r\n\r\n",
        )
        .await;
        assert_eq!(param.body_string(), "param-3;<rest>=deep/leaf");
    }
}
