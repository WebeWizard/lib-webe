# Quickstart & Validation Guide: Revamp Webe Web

**Feature**: 002-revamp-webe-web | **Date**: 2026-06-14

This guide validates the revamped `webe_web` crate end to end from a clean checkout.
It references [contracts/public-api.md](contracts/public-api.md) and
[data-model.md](data-model.md) instead of duplicating behavior detail.

## Prerequisites

- Toolchain pinned by `rust-toolchain.toml` (stable, edition 2024, MSRV 1.85).
- No external system libraries required (the `webe_web` default build is
  self-contained; `webe_auth` MySQL features stay out of scope here).

## Build & lint gates (Constitution I)

```bash
cargo fmt --check
cargo clippy -p webe_web -- -D warnings
cargo build -p webe_web
```

Expected: formatting clean, zero clippy warnings, successful build with no `dbg!`
output and no dead `processor` enum.

## Automated verification (FR-017, FR-018, SC-002, SC-007)

```bash
# Crate-local unit + integration tests for the supported scope
cargo test -p webe_web

# Whole-workspace, including the rewritten facade-level http test
cargo test --workspace
```

Expected: all tests pass; **no `#[ignore]` remains** on the supported server
workflow (the old `tests/http.rs` ignore is removed). Test files map to behavior:

| Test file | Covers |
|-----------|--------|
| `crates/webe_web/tests/routing.rs` | exact/param/terminal selection; `404` vs `405` |
| `crates/webe_web/tests/request_parsing.rs` | line/header limits; `505` version; framing `400` |
| `crates/webe_web/tests/body.rs` | `Content-Length` + `chunked` bodies; rejections |
| `crates/webe_web/tests/responses.rs` | status/headers/framing; keep-alive vs close |
| `crates/webe_web/tests/responders.rs` | built-in responder success + failure paths |
| `tests/http.rs` | facade-level async server smoke test |

## Manual scenario walkthrough (maps to Acceptance Scenarios)

Run the example server:

```bash
cargo run -p basic_server -- --bind-ip 127.0.0.1 --bind-port 8080
```

Then exercise the supported behaviors (US1–US3):

1. **Successful match (US1-1)** — `curl -i http://127.0.0.1:8080/` returns the
   static responder's `200` status, headers, and body.
2. **Not found (US1-2)** — request an unregistered path → documented `404`; server
   stays up.
3. **Method not allowed (US1-3)** — `curl -i -X DELETE http://127.0.0.1:8080/`
   (path matches a route, method does not) → `405`.
4. **Bad request (US1-4)** — send a malformed request line (e.g. via
   `printf 'GET\r\n\r\n' | nc 127.0.0.1 8080`) → documented `400`; server stays up.
5. **Routing determinism (US2)** — register exact + parameterized routes and confirm
   the exact route and most-specific parameterized route win as documented.
6. **Body handling (US3-1)** — POST a `Content-Length` body and a
   `Transfer-Encoding: chunked` body; both are read without buffering the whole body.
7. **Unsupported framing (US3-2)** — send a request with both `Content-Length` and
   `Transfer-Encoding` → `400`; responder is not invoked.
8. **Keep-alive (SC-004)** — send two sequential requests on one connection
   (`curl -i http://127.0.0.1:8080/ http://127.0.0.1:8080/`); both get correct,
   self-delimited responses; a `Connection: close` request closes afterward.

## Documentation check (FR-015, FR-020, SC-001, SC-008)

- `crates/webe_web/README.md` exists and states the supported HTTP/1.1 subset, the
  explicit exclusions, a usage example, and migration notes for the breaking changes
  (error consolidation, response framing, `Route`/`RouteMap` module move).
- A developer can stand up a server with one static route and one file-backed route
  using only the README and example (SC-001).

## Done criteria

- All gate commands above pass.
- Every documented supported behavior has at least one automated test (SC-002).
- No stale/ignored verification paths remain for the supported workflow (SC-007).
- No undocumented public behavior inside the supported scope (SC-008).
