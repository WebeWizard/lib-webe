# Phase 1 Data Model: Revamp Webe Web

**Feature**: 002-revamp-webe-web | **Date**: 2026-06-14

Entities are the in-memory types the crate exposes or operates on. This revamp
reorganizes where they live (see [plan.md](plan.md)) but keeps them within the
existing supported HTTP/1.1 subset. Fields marked *(new)* are added by the revamp;
the rest already exist in `crates/webe_web/src/`.

## Server

Represents a bound HTTP server that accepts connections and dispatches supported
requests. Lives in `server.rs` (slimmed to lifecycle only).

| Field | Type | Notes |
|-------|------|-------|
| `ip` | `Ipv4Addr` | Configured bind address (FR-001) |
| `port` | `u16` | Configured bind port (FR-001) |
| `listener` | `TcpListener` | tokio listener created at bind |

- **Lifecycle**: `new(ip, port) -> Result<Server, WebError>` binds (bind failure is
  a typed, actionable `WebError`); `start(routes) -> Result<(), WebError>` accepts
  connections and spawns a per-connection task that runs the processor loop.
- **Validation**: bind success/failure are documented outcomes (FR-001); accept
  failures are surfaced as typed server errors, not panics.

## Route

Represents a method + path pattern and how it competes with other routes. Moves to
`route.rs`.

| Field | Type | Notes |
|-------|------|-------|
| `method` | `String` | Uppercased HTTP method |
| `uri` | `String` | Normalized to a leading `/` (FR-003) |
| `has_params` | `bool` | True if the pattern contains `<param>` segments |

- **Validation**: registration normalizes paths so leading-slash and no-leading-slash
  declarations are equivalent (FR-003).
- **Relationships**: keyed inside `RouteMap`; selected by the matching rules below.

## Route Map

The collection of registered routes and their responders. Moves to `route.rs`.

| Field | Type | Notes |
|-------|------|-------|
| `inner` | `HashMap<Route, Box<dyn Responder>>` | Registered routes → responders |

- **Operations**: `add_route(route, responder)` (normalizes path), and matching:
  - `find_path_matches(request) -> set of routes whose path pattern matches` *(new, extracted)*
  - `find_best_route(request) -> Option<&Route>` selects among method-matching routes
  - **Selection order** (FR-004, deterministic):
    1. Exact non-parameterized match wins over any parameterized match.
    2. Among parameterized matches, the one with the most matching path parts wins;
       ties broken by the earliest wildcard position (most specific). Terminal
       parameterized routes capture the remainder of the path.
  - **405 vs 404** *(new)*: if path-pattern matches exist but none match the method →
    `405`; if no path-pattern matches → `404` (FR-002, FR-004).

## Route Parameter

A captured value from a parameterized path supplied to a responder. Produced by
`parse_route_params` in `route.rs`.

| Field | Type | Notes |
|-------|------|-------|
| name | `String` | The `<name>` token from the route pattern (FR-005) |
| value | `String` | Captured segment; terminal params capture the joined remainder |

- Exposed to responders as `&Vec<(String, String)>` for non-terminal and terminal
  captures (FR-005).

## Request

A parsed client request within the supported scope. Stays in `request.rs`.

| Field | Type | Notes |
|-------|------|-------|
| `total_size` | `usize` | Accumulated parsed size, bounded by limits (FR-006) |
| `method` | `String` | Uppercased method |
| `uri` | `String` | Request target path |
| `version` | `String` | Must equal `HTTP/1.1`, else `505` *(new validation)* |
| `headers` | `Option<HashMap<String, String>>` | Lowercased names, comma-combined values (FR-008) |
| `message_body` | `Option<Pin<Box<dyn AsyncBufRead + Send + Sync>>>` | Framed body reader assigned by the body module |

- **Validation rules** (all before responder invocation, FR-007):
  - Request line must have exactly 3 parts; else `400`.
  - Request-line size > `MAX_REQUEST_LINE_SIZE` → `400`/URI-too-large outcome.
  - Header section size > `MAX_HEADERS_SIZE` → documented client error.
  - Header line missing `:` separator → `400`.
  - `version != "HTTP/1.1"` → `505` *(new)*.
  - Both `Content-Length` and `Transfer-Encoding` present → `400` *(new)*.
  - Unsupported transfer coding (final coding not `chunked`) → rejected *(new validation path)*.
- **Header combining**: duplicate names combined with commas, framing headers
  excepted and routed through framing validation (FR-008).

## Body Framing *(new module `body.rs`)*

Not a stored struct but the decision logic that assigns `Request.message_body` and
chooses `Response` framing.

- **Request framing inputs**: `Content-Length` (single, parseable `u64`) or
  `Transfer-Encoding` ending in `chunked`.
- **Supported request framings** (FR-009): `Content-Length` body (read exactly N
  bytes); `chunked` body (decoded via `ChunkedDecoder`); no body.
- **Rejected**: both headers present (`400`); non-final or unknown transfer codings;
  unparseable `Content-Length`.
- **Response framing outputs** (Decision 1): known length → `Content-Length`;
  streamed unknown length → `Transfer-Encoding: chunked` via the new chunked encoder;
  no body → neither.

## Response

The status, headers, optional body, and connection preference sent to the client.
Stays in `response.rs`.

| Field | Type | Notes |
|-------|------|-------|
| `status` | `Status` | Status line code + reason (FR-010) |
| `keep_alive` | `bool` | Reconciled against framing + request `Connection` (FR-010, SC-004) |
| `headers` | `HashMap<String, String>` | Response headers |
| `message_body` | `Option<Pin<Box<dyn AsyncBufRead + Send>>>` | Streamed body |

- **State transitions when written** (`respond`):
  1. Write `HTTP/1.1 <code> <reason>`.
  2. Determine framing (Decision 1) and set `Content-Length` or chunked header.
  3. Reconcile `Connection` header from `keep_alive` (kept-alive only when body is
     self-delimiting).
  4. Write headers, blank line, then framed body bytes.
  - A body-reader failure mid-write surfaces a typed response/write error (FR-014).

## Responder

Developer-provided or built-in request handling. Trait in `responders/mod.rs`.

- `validate(request, params, validation) -> ValidationResult` — may accept, reject
  with a `Status`, or forward a wrapped validation (FR-011).
- `build_response(request, params, validation) -> Result<Response, u16>` — returns a
  response or a fallback status code (FR-011).
- **States**: validation-success, validation-failure, build-success,
  build-failure — each independently testable (FR-011, FR-017).

## Built-In Responder

Included responder behavior (FR-012). Each documents its success and failure paths.

| Responder | Module | Behavior | Key failure paths |
|-----------|--------|----------|-------------------|
| `StaticResponder` | `static_message.rs` | Fixed status + message body | n/a (always responds) |
| `FileResponder` | `file.rs` | Serves files from a mounted dir | path traversal → `403`/`404`; directory; missing file; index; symlink; write attempts (FR-013) |
| `OptionsResponder` | `options.rs` | CORS preflight response | configured origin/method/header echo |
| `SpaResponder` | `spa.rs` | Single-page-app fallback to an index | missing index file |

- **FileResponder safety** (FR-013): all resolved paths MUST stay within the
  configured mount point; traversal outside is denied with a documented status;
  missing files, directories, index files, symlink targets, and write attempts each
  have a deterministic documented outcome.

## Failure (`WebError`) *(new consolidated type in `error.rs`)*

Typed, matchable failure categories surfaced to developers and mapped to documented
client responses (FR-014, SC-003).

| Category | Wraps / cause | Client mapping |
|----------|---------------|----------------|
| `Bind` | `std::io::Error` at bind | server setup failure (not client-visible) |
| `Accept` | `std::io::Error` at accept | server failure (not client-visible) |
| `Request` | malformed line/headers, limits | `400` / size-specific |
| `Version` | non-`HTTP/1.1` | `505` |
| `Body` | framing conflict / unsupported coding / bad length | `400` |
| `Routing` | no path match / method mismatch | `404` / `405` |
| `Response` | body read or socket write failure | connection-level / `500` |
| `Responder` | validation or build failure | responder-provided status |

- Existing `RequestError`, `ResponseError`, `ServerError` get `From` conversions into
  `WebError` so current call sites keep working while gaining a categorized surface.

## Supported HTTP Scope

A documentation entity (README + crate docs), not a runtime type. Records the
committed subset: `HTTP/1.1` only; `Content-Length` and final-`chunked` body
framing; exact/parameterized/terminal routing with `404`/`405`; comma-combined
headers; the four built-in responders. Lists notable exclusions (e.g. HTTP/1.0,
HTTP/2, content codings like gzip/deflate, `Expect: 100-continue`, trailers,
multipart) without promising completeness (FR-015, SC-008).
