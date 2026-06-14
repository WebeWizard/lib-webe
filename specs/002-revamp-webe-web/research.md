# Phase 0 Research: Revamp Webe Web

**Feature**: 002-revamp-webe-web | **Date**: 2026-06-14

This document resolves the design decisions needed to plan the revamp. Every
decision stays inside the crate's existing supported scope (FR-016): no new
HTTP/1.1 capability is added unless it is required to make an already-supported
behavior correct, documented, and testable. Decisions are grounded in the current
code in `crates/webe_web/src/`.

## Decision 1 — Outgoing response body framing

**Decision**: Frame outgoing response bodies explicitly. Send a `Content-Length`
header when the body length is known up front; use `Transfer-Encoding: chunked`
when streaming a body whose length is not known up front; send neither for
bodyless responses. Reconcile `Connection: keep-alive`/`close` against the framing
so a kept-alive connection is only reused when the body is self-delimiting.

**Rationale**: Today `Response::respond()` writes the body bytes but emits no
`Content-Length` and no chunked framing, while defaulting `keep_alive = true` and
injecting `Connection: keep-alive`. On a reused connection the client cannot tell
where one response body ends and the next begins, which breaks SC-004 (sequential
keep-alive requests). Explicit framing is the minimum fix that makes keep-alive
correct. Chunked encoding additionally preserves streaming without buffering the
whole body in memory (SC-005, Constitution IV), and the crate already commits to
chunked as a supported coding (FR-009), so this reuses an in-scope capability
rather than adding a new one.

**Alternatives considered**:

- *Always buffer the body and send `Content-Length` only.* Rejected: buffering the
  entire body violates SC-005 and Constitution IV for large bodies, and the file
  responder streams from disk.
- *Delimit bodies by closing the connection (no framing).* Rejected: effectively
  disables keep-alive for every bodied response, directly defeating SC-004.

## Decision 2 — `405 Method Not Allowed` vs `404 Not Found`

**Decision**: When a request path matches a registered route pattern (exact or
parameterized) but no route for that path matches the request method, respond
`405 Method Not Allowed`. When no registered route pattern matches the path at all,
respond `404 Not Found`. Route matching first determines the set of routes whose
path pattern matches; if that set is non-empty but none share the method, it is a
405; if empty, it is a 404.

**Rationale**: Required explicitly by FR-002, FR-004, and Acceptance Scenario 3.
Today `find_best_route` filters by method first, so a path that exists under a
different method falls through to the 404 branch — incorrect per the spec. The fix
is to separate path-pattern matching from method matching in `route.rs`, which is a
behavior correction on existing routing, not a new protocol feature.

**Alternatives considered**:

- *Keep returning `404` for method mismatch.* Rejected: contradicts the clarified
  spec requirement.
- *Include an `Allow` header listing permitted methods.* Deferred: not required by
  the spec for this revamp; the `405` status is the committed behavior. May be noted
  as an exclusion in the README.

## Decision 3 — Accept only `HTTP/1.1`; reject other versions with `505`

**Decision**: Parse the request-line version token and accept only `HTTP/1.1`. Any
other version (e.g. `HTTP/1.0`, `HTTP/2`) is rejected with
`505 HTTP Version Not Supported` before any responder runs.

**Rationale**: Clarified requirement (FR-006, FR-007, Edge Cases). Today the
version string is parsed into `Request.version` but never validated, and responses
are hardcoded to `HTTP/1.1`. Enforcing the version makes the request/response
contract honest and is validated at the request boundary in `request.rs`.

**Alternatives considered**:

- *Accept any version and always answer `HTTP/1.1`.* Rejected: silently mismatches
  the client's protocol expectation and contradicts the clarified spec.

## Decision 4 — Reject conflicting `Content-Length` + `Transfer-Encoding` with `400`

**Decision**: If a request carries both `Content-Length` and `Transfer-Encoding`,
reject it with `400 Bad Request` before invoking a responder. Supported request
body framing is exactly: a single `Content-Length`, or a `Transfer-Encoding` whose
final coding is `chunked`. Any other transfer coding is rejected.

**Rationale**: Clarified requirement (FR-007, FR-009). Today `server.rs` processes
`transfer-encoding` and then independently processes `content-length`, so a request
with both is mishandled instead of rejected. Centralizing the framing decision in
`body.rs` makes the supported-vs-rejected boundary explicit and testable, and
removes request smuggling ambiguity (a security concern aligned with Constitution
II/IV).

**Alternatives considered**:

- *Prefer one header and ignore the other.* Rejected: ambiguous framing is a
  request-smuggling risk and the spec mandates rejection.

## Decision 5 — Duplicate header combining

**Decision**: Combine duplicate request headers (case-insensitive name) into a
single comma-joined value exposed to responders, **except** body-framing headers
(`Content-Length`, `Transfer-Encoding`), which are validated by the stricter
framing rules in Decision 4 rather than blindly combined.

**Rationale**: Clarified requirement (FR-008). The current `read_headers` already
lowercases names and comma-combines duplicates, so this is mostly preserving and
documenting existing behavior, plus carving out body-framing headers so a duplicate
`Content-Length` is treated as a framing conflict (`400`) rather than a combined
value.

**Alternatives considered**:

- *Combine all headers uniformly including framing headers.* Rejected: would let a
  conflicting/duplicate `Content-Length` slip past framing validation.

## Decision 6 — Module reorganization boundary

**Decision**: Split `server.rs` into `route.rs` (Route, RouteMap, `find_best_route`,
`parse_route_params`, 405/404 logic), a slimmed `server.rs` (bind/accept/start),
and `processor.rs` (the per-connection request lifecycle, repurposing today's dead
`ProcessError` enum module). Add `error.rs` for a consolidated typed error and
`body.rs` for request/response framing. Add a chunked encoder under `encoding/`.

**Rationale**: The current `server.rs` mixes three responsibilities (routing,
server lifecycle, request loop) in one file, making the request lifecycle hard to
test in isolation and obscuring where framing/validation decisions live. Separating
them supports FR-014 (typed failures), FR-017 (targeted tests per behavior), and
Constitution I (self-contained, maintainable modules). `processor.rs` already
exists but is dead; repurposing it avoids inventing a new module name.

**Alternatives considered**:

- *Leave `server.rs` as one file and only harden behavior.* Rejected: the monolith
  is the main maintainability obstacle the revamp targets, and per-behavior tests
  (FR-017) are much harder against an entangled request loop.

## Decision 7 — Error consolidation strategy

**Decision**: Introduce a single crate error taxonomy in `error.rs` that categorizes
failures as server/bind, request/parse, body/framing, routing, response/write, and
responder failures, with `From` conversions from the existing `RequestError`,
`ResponseError`, and `ServerError` so each client-visible failure maps to a
documented status and each developer-visible failure names its category (FR-014,
SC-003). Internal responder fallbacks continue to surface a status code.

**Rationale**: Today failures are spread across `RequestError`, `ResponseError`,
`ServerError`, the dead `ProcessError`, and bare `u16` codes from responders, with
several paths collapsing distinct failures into a generic `400`/`InternalError`.
A categorized taxonomy makes failures matchable and actionable without changing the
responder trait's `Result<Response, u16>` ergonomics that the built-in responders
rely on.

**Alternatives considered**:

- *Adopt `thiserror`/`anyhow`.* Rejected for now: adds a dependency where a small
  hand-written enum suffices and keeps the default build dependency-light
  (Constitution Additional Constraints). May be revisited but not required.

## Summary of resolved unknowns

| Topic | Resolution | Drives |
|-------|-----------|--------|
| Response body framing | `Content-Length` known / `chunked` streamed / none for bodyless | FR-010, SC-004, SC-005 |
| Method mismatch | `405` when path matches but method does not, else `404` | FR-002, FR-004 |
| HTTP version | Accept `HTTP/1.1` only, else `505` | FR-006, FR-007 |
| Conflicting framing headers | Reject both-present with `400` | FR-007, FR-009 |
| Duplicate headers | Comma-combine, framing headers excepted | FR-008 |
| Module layout | route / server / processor split + error + body | FR-014, FR-017 |
| Error model | Hand-written categorized taxonomy, no new deps | FR-014, SC-003 |

All NEEDS CLARIFICATION items are resolved. No decision expands the crate beyond its
existing supported HTTP/1.1 subset.
