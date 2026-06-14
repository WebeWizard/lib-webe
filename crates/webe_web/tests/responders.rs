//! Integration tests for the built-in responders and file-serving safety (US4).

mod common;

use std::path::PathBuf;

use common::{TestClient, spawn_server};
use webe_web::responders::file::FileResponder;
use webe_web::responders::options::OptionsResponder;
use webe_web::responders::spa::SpaResponder;
use webe_web::responders::static_message::StaticResponder;
use webe_web::server::{Route, RouteMap};

/// Creates a clean temporary mount directory unique to this test.
fn temp_mount(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("webe_web_test_{}_{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp mount");
    dir
}

// ---------- StaticResponder ----------

#[tokio::test]
async fn static_responder_returns_its_message() {
    let mut map = RouteMap::new();
    map.add_route(
        Route::new("GET", "/static"),
        StaticResponder::new(200, "hi there".to_string()),
    );
    let addr = spawn_server(map).await;

    let response =
        TestClient::request(addr, b"GET /static HTTP/1.1\r\nConnection: close\r\n\r\n").await;
    assert_eq!(response.status, 200);
    assert_eq!(response.body_string(), "hi there");
}

// ---------- OptionsResponder (CORS preflight) ----------

#[tokio::test]
async fn options_responder_echoes_preflight_headers() {
    let mut map = RouteMap::new();
    map.add_route(
        Route::new("OPTIONS", "/<dump>"),
        OptionsResponder::new(
            "http://localhost:1234".to_owned(),
            "POST, GET, OPTIONS".to_owned(),
            "content-type".to_owned(),
        ),
    );
    let addr = spawn_server(map).await;

    let response = TestClient::request(
        addr,
        b"OPTIONS /anything HTTP/1.1\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_eq!(response.status, 204);
    assert_eq!(
        response.header("Access-Control-Allow-Origin"),
        Some(&"http://localhost:1234".to_string())
    );
    assert_eq!(
        response.header("Access-Control-Allow-Methods"),
        Some(&"POST, GET, OPTIONS".to_string())
    );
    assert_eq!(
        response.header("Access-Control-Allow-Headers"),
        Some(&"content-type".to_string())
    );
}

// ---------- FileResponder success ----------

#[tokio::test]
async fn file_responder_serves_an_existing_file() {
    let mount = temp_mount("file_serve");
    std::fs::write(mount.join("hello.txt"), b"file contents").unwrap();

    let mut map = RouteMap::new();
    map.add_route(
        Route::new("GET", "/<path>"),
        FileResponder::new(mount.to_string_lossy().into_owned(), "<path>".to_owned())
            .expect("file responder"),
    );
    let addr = spawn_server(map).await;

    let response = TestClient::request(
        addr,
        b"GET /hello.txt HTTP/1.1\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_eq!(response.status, 200);
    assert_eq!(response.body_string(), "file contents");

    let _ = std::fs::remove_dir_all(&mount);
}

// ---------- SpaResponder ----------

#[tokio::test]
async fn spa_responder_falls_back_to_index() {
    let mount = temp_mount("spa_ok");
    std::fs::write(mount.join("index.html"), b"<html>spa</html>").unwrap();

    let mut map = RouteMap::new();
    map.add_route(
        Route::new("GET", "/<path>"),
        SpaResponder::new(
            mount.to_string_lossy().into_owned(),
            "index.html".to_owned(),
        )
        .expect("spa responder"),
    );
    let addr = spawn_server(map).await;

    // a deep client-side route still serves the app index
    let response = TestClient::request(
        addr,
        b"GET /flash/23434455 HTTP/1.1\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_eq!(response.status, 200);
    assert_eq!(response.body_string(), "<html>spa</html>");

    let _ = std::fs::remove_dir_all(&mount);
}

#[tokio::test]
async fn spa_responder_missing_index_is_not_found() {
    let mount = temp_mount("spa_missing");

    let mut map = RouteMap::new();
    map.add_route(
        Route::new("GET", "/<path>"),
        SpaResponder::new(
            mount.to_string_lossy().into_owned(),
            "index.html".to_owned(), // does not exist
        )
        .expect("spa responder"),
    );
    let addr = spawn_server(map).await;

    let response =
        TestClient::request(addr, b"GET /anything HTTP/1.1\r\nConnection: close\r\n\r\n").await;
    assert_eq!(response.status, 404);

    let _ = std::fs::remove_dir_all(&mount);
}

// ---------- FileResponder safety (FR-013) ----------

#[tokio::test]
async fn file_responder_denies_path_traversal() {
    let mount = temp_mount("traversal");
    std::fs::write(mount.join("ok.txt"), b"ok").unwrap();

    let mut map = RouteMap::new();
    map.add_route(
        Route::new("GET", "/<path>"),
        FileResponder::new(mount.to_string_lossy().into_owned(), "<path>".to_owned())
            .expect("file responder"),
    );
    let addr = spawn_server(map).await;

    let response = TestClient::request(
        addr,
        b"GET /../../../../etc/passwd HTTP/1.1\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_eq!(response.status, 404);

    let _ = std::fs::remove_dir_all(&mount);
}

#[tokio::test]
async fn file_responder_directory_without_index_is_not_found() {
    let mount = temp_mount("dir");
    std::fs::create_dir_all(mount.join("sub")).unwrap();

    let mut map = RouteMap::new();
    map.add_route(
        Route::new("GET", "/<path>"),
        FileResponder::new(mount.to_string_lossy().into_owned(), "<path>".to_owned())
            .expect("file responder"),
    );
    let addr = spawn_server(map).await;

    let response =
        TestClient::request(addr, b"GET /sub HTTP/1.1\r\nConnection: close\r\n\r\n").await;
    assert_eq!(response.status, 404);

    let _ = std::fs::remove_dir_all(&mount);
}

#[tokio::test]
async fn file_responder_missing_file_is_not_found() {
    let mount = temp_mount("missing");

    let mut map = RouteMap::new();
    map.add_route(
        Route::new("GET", "/<path>"),
        FileResponder::new(mount.to_string_lossy().into_owned(), "<path>".to_owned())
            .expect("file responder"),
    );
    let addr = spawn_server(map).await;

    let response =
        TestClient::request(addr, b"GET /nope.txt HTTP/1.1\r\nConnection: close\r\n\r\n").await;
    assert_eq!(response.status, 404);

    let _ = std::fs::remove_dir_all(&mount);
}

#[cfg(unix)]
#[tokio::test]
async fn file_responder_denies_symlink_outside_mount() {
    let mount = temp_mount("symlink");
    // a symlink inside the mount pointing outside of it
    let link = mount.join("escape");
    std::os::unix::fs::symlink("/etc/passwd", &link).unwrap();

    let mut map = RouteMap::new();
    map.add_route(
        Route::new("GET", "/<path>"),
        FileResponder::new(mount.to_string_lossy().into_owned(), "<path>".to_owned())
            .expect("file responder"),
    );
    let addr = spawn_server(map).await;

    let response =
        TestClient::request(addr, b"GET /escape HTTP/1.1\r\nConnection: close\r\n\r\n").await;
    assert_eq!(response.status, 404);

    let _ = std::fs::remove_dir_all(&mount);
}

#[tokio::test]
async fn file_responder_put_to_directory_is_rejected() {
    let mount = temp_mount("put_dir");
    std::fs::create_dir_all(mount.join("adir")).unwrap();

    let mut map = RouteMap::new();
    map.add_route(
        Route::new("PUT", "/<path>"),
        FileResponder::new(mount.to_string_lossy().into_owned(), "<path>".to_owned())
            .expect("file responder"),
    );
    let addr = spawn_server(map).await;

    let response = TestClient::request(
        addr,
        b"PUT /adir HTTP/1.1\r\nContent-Length: 3\r\nConnection: close\r\n\r\nabc",
    )
    .await;
    assert_eq!(response.status, 404);

    let _ = std::fs::remove_dir_all(&mount);
}
