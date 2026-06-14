# Implementation Plan: Revamp Webe Web

**Branch**: `002-revamp-webe-web` | **Date**: 2026-06-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/002-revamp-webe-web/spec.md`

## Summary

Revamp the `webe_web` crate into a reliable, well-documented, well-tested HTTP/1.1
server library **within its existing scope** — it is explicitly not feature
complete and this revamp adds no new protocol features except where one is
required to make an already-supported behavior correct and testable (FR-016).

The work is primarily a **reorganization plus hardening** pass:

- Split the `server.rs` monolith (which today mixes routing, server lifecycle, and
  the per-connection request loop) into focused modules: `route` (route + matching
  + params), `server` (bind/accept/start), and `processor` (the connection request
  lifecycle — repurposing today's dead `processor.rs` enum).
- Introduce typed, categorized failures (`error.rs`) so server, request, response,
  route, and responder problems are matchable and actionable (FR-014).
- Centralize request body framing and response body framing in a `body` module so
  the supported framings (`Content-Length`, final `Transfer-Encoding: chunked`) and
  their rejections (conflicting both headers → `400`; non-`HTTP/1.1` → `505`;
  unsupported transfer codings → `400`/`501`) live in one validated place
  (FR-007, FR-008, FR-009).
- Make outgoing responses self-delimiting so keep-alive is correct: send
  `Content-Length` when the body length is known, `Transfer-Encoding: chunked`
  when streaming an unknown-length body, and neither for bodyless responses
  (resolves the response-framing gap behind FR-010 and SC-004; reuses the chunked
  support already committed in FR-009 and preserves streaming per SC-005).
- Add the missing crate `README.md` and crate-local integration tests, and rewrite
  the stale, ignored workspace `tests/http.rs` against the async API so the
  documented workflow is fully covered by automated tests (FR-015, FR-017, FR-018,
  SC-002, SC-007).

The crate stays reachable through the `webe::web` facade re-export (FR-019), and
intentional breaking changes (error type consolidation, response framing, module
paths) are documented in the README before release (FR-020).

## Technical Context

**Language/Version**: Rust (stable channel per `rust-toolchain.toml`, edition 2024, MSRV 1.85)

**Primary Dependencies**: `tokio` (async runtime + net + io), `async-trait` (dyn `Responder`), `limit_read`, `pin-project-lite`, `serde`/`serde_json` (existing). No new runtime dependencies planned; chunked response encoding is implemented in-crate alongside the existing chunked decoder.

**Storage**: N/A (static file responder reads from a configured mount directory only)

**Testing**: `cargo test` — unit tests in `src/` under `#[cfg(test)]`, integration tests under `crates/webe_web/tests/`, plus the rewritten workspace-level `tests/http.rs` covering the facade path

**Target Platform**: Cross-platform async Rust library (Linux/macOS/Windows) on a tokio runtime

**Project Type**: Single Rust library crate within the existing Cargo workspace, surfaced via the `webe` facade

**Performance Goals**: Route matching deterministic and acceptable across ≥25 mixed exact/parameterized routes (SC-006); request and response bodies streamed without buffering the whole body in memory at the documented max accepted size (SC-005); no O(n²)-or-worse hot paths over request size or route-table size (Constitution IV)

**Constraints**: No `.unwrap()`/`.expect()` in library paths; typed categorized errors; no `unsafe`; lint-clean under `cargo fmt --check` and `cargo clippy` (warnings = errors); default build needs no external system libraries; public items carry doc comments; blocking work kept off the async executor; remove stray debug output (`dbg!`) from request parsing

**Scale/Scope**: Small-to-medium async crate — Server, Route/RouteMap, Request, Response, Responder trait, four built-in responders (static message, file, options/preflight, SPA fallback), chunked codec, and a consolidated error type; documented supported HTTP/1.1 subset with explicit exclusions

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Evaluated against `.specify/memory/constitution.md` v1.0.0:

- **I. Code Quality**: PASS (planned). The revamp adds doc comments to all public
  items (Server, Route, RouteMap, Request, Response, Responder, built-in
  responders, error type), replaces ad-hoc/`String`-and-`u16` error paths with a
  typed categorized error (FR-014), removes the `dbg!` call and dead
  `processor.rs` enum, keeps `webe_web` free of workspace-internal deps, and
  targets `cargo fmt`/`clippy` clean with no `unsafe`.
- **II. Testing Standards (NON-NEGOTIABLE)**: PASS (planned). Every documented
  supported behavior maps to a test (FR-017, SC-002), tests are added in the same
  change set and observed Red→Green, success **and** failure paths are covered for
  request parsing, routing, body handling, response writing, keep-alive/close, and
  built-in responders, and the stale ignored `tests/http.rs` is rewritten so
  `cargo test --workspace` runs the documented workflow without manual request
  crafting (FR-018, SC-007). No ignored/commented-out tests remain for supported
  scope.
- **III. User Experience Consistency**: PASS (planned). Errors are actionable and
  name the failure category (FR-014, SC-003); async/naming conventions stay
  consistent with the other crates; the crate remains reachable via `webe::web`
  (FR-019); breaking changes are documented in the crate README with migration
  guidance before release (FR-020); the `basic_server` example continues to
  demonstrate the documented happy path and is covered by automated tests (FR-018).
- **IV. Performance Requirements**: PASS (planned). Request and response bodies are
  streamed (chunked/`Content-Length`) rather than fully buffered (SC-005,
  Constitution IV); route matching avoids worse-than-linear scans per request over
  the route table and is verified deterministic at ≥25 routes (SC-006); no blocking
  calls are introduced on async paths.

Initial gate: **PASS**. No violations to justify; Complexity Tracking left empty.

**Open design decisions resolved in `research.md`** (none block the gate; all stay
within existing crate scope per FR-016): outgoing response body framing strategy,
method-not-allowed (`405`) detection vs not-found (`404`), and `HTTP/1.1`-only
version enforcement (`505`).

## Project Structure

### Documentation (this feature)

```text
specs/002-revamp-webe-web/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output — resolved design decisions
├── data-model.md        # Phase 1 output — entities + state/transitions
├── quickstart.md        # Phase 1 output — runnable validation guide
├── contracts/           # Phase 1 output
│   └── public-api.md     # Public API + HTTP behavior contract for the crate
├── checklists/
│   └── requirements.md  # Spec quality checklist (already passing)
└── tasks.md             # Phase 2 output (/speckit.tasks — NOT created by /speckit.plan)
```

### Source Code (repository root)

Current `crates/webe_web/src/` layout (before revamp):

```text
crates/webe_web/src/
├── constants.rs
├── encoding/
│   ├── chunked.rs        # ChunkedDecoder
│   └── mod.rs
├── lib.rs                # module declarations only
├── processor.rs          # DEAD: only an unused `ProcessError` enum
├── request.rs            # request line + header parsing (+ stray dbg!)
├── responders/
│   ├── file.rs
│   ├── mod.rs            # Responder trait
│   ├── options.rs
│   ├── spa.rs
│   └── static_message.rs
├── response.rs           # Response + respond() (no body framing)
├── server.rs             # MONOLITH: Route, RouteMap, Server, matching,
│                         # param parsing, and the whole process_stream loop
├── status.rs
└── validation.rs
```

Target `crates/webe_web/src/` layout (after revamp — reorganized):

```text
crates/webe_web/
├── Cargo.toml
├── README.md             # NEW: supported HTTP/1.1 subset, exclusions, usage,
│                         # migration notes for breaking changes (FR-015/020)
├── src/
│   ├── lib.rs            # module declarations + crate-level docs + re-exports
│   ├── constants.rs      # size limits + MIME map (unchanged)
│   ├── error.rs          # NEW: consolidated typed WebError categories (FR-014)
│   ├── status.rs         # Status (unchanged; add docs)
│   ├── validation.rs     # Validation / ValidationResult (add docs)
│   ├── request.rs        # Request parsing: line, version check, headers,
│   │                     # header combining; dbg! removed (FR-006/007/008)
│   ├── response.rs       # Response + framed writing (Content-Length/chunked,
│   │                     # keep-alive reconciliation) (FR-010)
│   ├── body.rs           # NEW: request/response body framing decisions —
│   │                     # Content-Length vs chunked vs reject (FR-007/008/009)
│   ├── route.rs          # NEW (extracted from server.rs): Route, RouteMap,
│   │                     # find_best_route, parse_route_params, 405-vs-404
│   ├── server.rs         # SLIMMED: Server bind/accept/start lifecycle only
│   ├── processor.rs      # REPURPOSED: per-connection request lifecycle
│   │                     # (former process_stream), keep-alive loop, dispatch
│   ├── encoding/
│   │   ├── mod.rs
│   │   ├── chunked.rs    # ChunkedDecoder (request bodies)
│   │   └── chunked_encoder.rs # NEW: chunked encoder for streamed responses
│   └── responders/
│       ├── mod.rs        # Responder trait (add docs)
│       ├── file.rs       # static files; document path-traversal/dir/symlink/missing
│       ├── options.rs    # preflight responder
│       ├── spa.rs        # single-page-app fallback
│       └── static_message.rs
└── tests/
    ├── routing.rs        # NEW: exact/param/terminal selection, 404 vs 405
    ├── request_parsing.rs# NEW: line/header limits, version 505, framing 400
    ├── body.rs           # NEW: Content-Length + chunked bodies, rejections
    ├── responses.rs      # NEW: status/headers/body framing, keep-alive vs close
    └── responders.rs     # NEW: built-in responder success + failure paths

tests/
└── http.rs               # REWRITTEN: stale ignored test replaced with async
                          # facade-level server smoke test (FR-018/SC-007)

examples/basic_server/
└── src/main.rs           # Updated only if public module paths/APIs move (FR-018)

src/lib.rs                # Workspace facade — `pub use webe_web as web` (unchanged)
```

**Structure Decision**: Single async library crate. The revamp stays inside
`crates/webe_web/` and keeps the `webe::web` facade re-export. The central
reorganization splits the `server.rs` monolith into `route.rs` (routing + matching
+ params), a slim `server.rs` (lifecycle), and `processor.rs` (the per-connection
request loop, repurposing today's dead module), and adds `error.rs` (typed
categories) and `body.rs` (framing). Unit tests live beside source under
`#[cfg(test)]`; supported-scope behavior is covered by crate integration tests in
`crates/webe_web/tests/`; and the previously ignored workspace `tests/http.rs` is
rewritten against the async API so `cargo test --workspace` exercises the facade
path. The `basic_server` example is updated only where public module paths change
and remains the primary developer-facing demonstration.

## Complexity Tracking

> No constitution violations. Section intentionally empty.
