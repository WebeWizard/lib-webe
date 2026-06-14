# webe_web

A small, asynchronous **HTTP/1.1** server library, built on [tokio]. It is reachable
on its own or through the `webe::web` facade in the parent `webe` crate.

`webe_web` is **not** a feature-complete HTTP implementation. It commits to a
documented subset of HTTP/1.1 (below) and aims to make that subset correct, tested,
and predictable rather than broad.

## Supported scope

- **Protocol**: `HTTP/1.1` request parsing and response writing.
- **Request bodies**: framed by a single `Content-Length`, or by
  `Transfer-Encoding` whose final coding is `chunked`, or no body.
- **Response bodies**: `Content-Length` when the length is known,
  `Transfer-Encoding: chunked` when streaming an unknown length, or neither when
  there is no body. Bodies are streamed, not fully buffered.
- **Routing**: exact, parameterized (`<name>`), and terminal-parameter routes with
  deterministic selection. A path match with no method match yields `405`; no path
  match yields `404`.
- **Headers**: duplicate request header names are lowercased and comma-combined
  (framing headers excepted).
- **Connections**: per-connection keep-alive when the response body is
  self-delimiting and the client did not request `Connection: close`.
- **Built-in responders**: `StaticResponder`, `FileResponder`, `OptionsResponder`,
  `SpaResponder`.

## Explicitly out of scope

The following are intentionally **not** implemented by this crate:

- HTTP/1.0 and HTTP/2 (non-`HTTP/1.1` versions are rejected with `505`).
- Content codings (gzip/deflate) and content negotiation.
- `Expect: 100-continue`, chunked trailers, and multipart parsing.
- TLS termination.
- Cookie/session handling (this lives in `webe_auth`).
- Any HTTP/1.1 feature not listed under **Supported scope**.

## Usage

```rust,no_run
use std::net::Ipv4Addr;

use webe_web::responders::static_message::StaticResponder;
use webe_web::server::{Route, RouteMap, Server};

#[tokio::main]
async fn main() -> Result<(), webe_web::error::WebError> {
    // Bind to 127.0.0.1:8080 (use port 0 to let the OS choose, then `local_addr()`).
    let server = Server::new(&Ipv4Addr::new(127, 0, 0, 1), &8080).await?;

    // Register routes against responders.
    let mut routes = RouteMap::new();
    routes.add_route(
        Route::new("GET", "/hello"),
        StaticResponder::new(200, "hello from webe_web".to_owned()),
    );

    // Run the accept loop (blocks the current task).
    server.start(routes).await
}
```

Through the parent facade the imports become `webe::web::server::{Route, RouteMap,
Server}` and `webe::web::responders::static_message::StaticResponder`.

## Errors

All public fallible operations surface the categorized [`error::WebError`]. Match on
it by category — `Bind`, `Accept`, `Request`, `Version`, `Body`, `Routing`,
`Response`, `Responder` — each of which `Display`s what failed and, where possible,
how to resolve it. Responders additionally use `ValidationResult` and
`Result<Response, u16>` as documented on the `Responder` trait.

## Migration notes

This revamp introduced a few breaking changes relative to the previous layout:

- **Error consolidation**: a single categorized `WebError` (in the new `error`
  module) is now returned from the server lifecycle and connection processing,
  replacing the former ad-hoc per-module error returns at the public boundary. The
  legacy `server::ServerError` is retained and converts into `WebError`.
- **`Route` / `RouteMap` moved**: routing types now live in a new `route` module.
  They are re-exported from `server` (`webe_web::server::{Route, RouteMap}`) so
  existing imports keep working.
- **Response framing**: responses are framed automatically from whether a body is
  present and whether its length is known (`Content-Length` vs chunked vs neither);
  you no longer set framing headers by hand for the supported cases.
- **`SPAResponder` → `SpaResponder`**: the single-page-application responder was
  renamed to match Rust naming conventions.

[tokio]: https://tokio.rs
