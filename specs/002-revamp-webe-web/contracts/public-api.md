# Public API & HTTP Behavior Contract: Webe Web

**Feature**: 002-revamp-webe-web | **Date**: 2026-06-14

This is the contract the revamped `webe_web` crate commits to, reachable via the
`webe::web` facade. It documents the public surface and the observable HTTP behavior
within the supported scope. "Supported" means committed and tested; anything not
listed is out of scope for this revamp (FR-015, FR-016).

## Module surface (`webe::web::*`)

| Path | Items |
|------|-------|
| `webe::web::server` | `Server`, and re-exports of `Route`, `RouteMap` |
| `webe::web::route` *(new)* | `Route`, `RouteMap` |
| `webe::web::request` | `Request`, `RequestError` |
| `webe::web::response` | `Response`, `ResponseError` |
| `webe::web::error` *(new)* | `WebError` (categorized) |
| `webe::web::status` | `Status` |
| `webe::web::validation` | `Validation`, `ValidationResult` |
| `webe::web::responders` | `Responder` trait |
| `webe::web::responders::static_message` | `StaticResponder` |
| `webe::web::responders::file` | `FileResponder` |
| `webe::web::responders::options` | `OptionsResponder` |
| `webe::web::responders::spa` | `SpaResponder` |

> Breaking change note: `Route`/`RouteMap` move from `server` to a new `route`
> module and are re-exported from `server` for source compatibility. The migration
> note in the README documents this (FR-020).

## Server contract

```text
Server::new(&Ipv4Addr, &u16) -> Result<Server, WebError>   // FR-001
Server::start(RouteMap<'static>) -> Result<(), WebError>    // FR-002
```

- Bind failure returns a typed `WebError::Bind` (no panic). Accept failure returns a
  typed server error. Each connection is handled on its own task.

## Routing contract (FR-002, FR-003, FR-004, FR-005)

- `Route::new(method, uri)` normalizes a missing leading `/`.
- Registration via `RouteMap::add_route(route, responder)`.
- **Selection is deterministic**:
  1. An exact (non-parameterized) route matching method + path wins.
  2. Otherwise the most specific parameterized route wins: most matching parts, then
     earliest wildcard. Terminal `<param>` captures the remaining path.
- **Parameters** are delivered to responders as `&Vec<(String, String)>` using the
  `<name>` token and captured value.
- **Method handling**: path pattern matches but method does not → `405`; no path
  pattern matches → `404`.

## Request contract (FR-006, FR-007, FR-008, FR-009)

Accepted:

- Request line `METHOD SP target SP HTTP/1.1 CRLF` within `MAX_REQUEST_LINE_SIZE`.
- Header block within `MAX_HEADERS_SIZE`; duplicate names combined with commas under
  a lowercased name (framing headers excepted).
- Body framed by a single `Content-Length`, or `Transfer-Encoding` whose final
  coding is `chunked`, or no body.

Rejected before any responder runs, with documented status:

| Condition | Status |
|-----------|--------|
| Malformed request line / missing header separator | `400 Bad Request` |
| Request line exceeds limit | `400` (URI-too-large outcome) |
| Header block exceeds limit | documented client error |
| HTTP version ≠ `HTTP/1.1` | `505 HTTP Version Not Supported` |
| Both `Content-Length` and `Transfer-Encoding` present | `400 Bad Request` |
| Unparseable `Content-Length` | `400 Bad Request` |
| Unsupported transfer coding (final ≠ `chunked`) | documented client error |

## Response contract (FR-010, SC-004, SC-005)

- Status line is always `HTTP/1.1 <code> <reason>`.
- **Body framing**: `Content-Length` when length is known; `Transfer-Encoding:
  chunked` when streaming unknown length; neither when there is no body.
- **Connection**: `keep-alive` only when the body is self-delimiting and the client
  did not request `close`; otherwise the connection closes after the response.
- Bodies are streamed, not fully buffered, up to the documented max accepted size.

## Responder contract (FR-011, FR-012, FR-013)

```text
async fn validate(&self, &Request, &Vec<(String,String)>, Validation) -> ValidationResult
async fn build_response(&self, &mut Request, &Vec<(String,String)>, Validation) -> Result<Response, u16>
```

- `validate` may accept, reject with a `Status`, or forward a wrapped validation.
- `build_response` returns a `Response` or a fallback status code that becomes a
  static error response.
- **Built-ins**: `StaticResponder`, `FileResponder`, `OptionsResponder`,
  `SpaResponder` each have a documented success path and relevant failure path.
- **`FileResponder` safety** (FR-013): never serves a path resolving outside the
  configured mount point; missing file, directory, index file, symlink target, and
  write attempts each have a deterministic documented status.

## Error contract (FR-014, SC-003)

- All public fallible operations return `Result<_, WebError>` (or, for responders,
  the existing `ValidationResult` / `Result<Response, u16>`).
- `WebError` is matchable by category: `Bind`, `Accept`, `Request`, `Version`,
  `Body`, `Routing`, `Response`, `Responder`.
- Each category's `Display` names what failed and, where possible, how to resolve it.

## Scope statement (FR-015, FR-016, SC-008)

**Supported**: `HTTP/1.1` request/response; `Content-Length` and final-`chunked`
body framing; exact/parameterized/terminal routing with `404`/`405`; comma-combined
request headers; the four built-in responders; bind and per-connection keep-alive.

**Explicitly out of scope (not added by this revamp)**: HTTP/1.0 and HTTP/2;
content codings (gzip/deflate); `Expect: 100-continue`; chunked trailers; multipart
parsing; TLS termination; cookie/session handling (lives in `webe_auth`); and any
HTTP/1.1 feature not listed under "Supported".
