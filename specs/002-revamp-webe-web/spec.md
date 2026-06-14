# Feature Specification: Revamp Webe Web

**Feature Branch**: `002-revamp-webe-web`

**Created**: 2026-06-13

**Status**: Draft

**Input**: User description: "Create a spec that revamps the webe_web crate. The crate aims to provide a http 1.1 server library. It is NOT feature complete so do not try to add missing http 1.1 features."

## Clarifications

### Session 2026-06-13

- Q: Which request body framing methods should the revamped `webe_web` spec commit to supporting? → A: Support `Content-Length` and final `Transfer-Encoding: chunked`; reject other body framing.
- Q: When a request path matches a registered route pattern but the HTTP method does not match any route for that path, what response should the spec require? → A: Return `405 Method Not Allowed` when the path matches a registered route pattern but the method does not; otherwise `404 Not Found`.
- Q: How should the server handle a request that includes both `Content-Length` and `Transfer-Encoding`? → A: Reject requests containing both `Content-Length` and `Transfer-Encoding` as `400 Bad Request`.
- Q: Which HTTP request versions should the revamped server accept within this spec? → A: Accept only `HTTP/1.1`; reject other versions with `505 HTTP Version Not Supported`.
- Q: How should duplicate request headers with the same name, including different casing, be represented to responders? → A: Combine duplicate request header values with commas under one case-insensitive header name, except body-framing headers are validated by their stricter rules.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Serve Basic HTTP Requests Reliably (Priority: P1)

A developer building a small server can register simple routes, accept supported HTTP/1.1 requests, and receive predictable responses for successful matches and common request failures.

**Why this priority**: This is the core value of the crate. Without a dependable request-to-response path, routing, responders, and documentation cannot be trusted.

**Independent Test**: Can be tested by starting a small server with a static route, sending supported requests over a local connection, and confirming the response status, headers, body, and connection behavior match the documented outcomes.

**Acceptance Scenarios**:

1. **Given** a server with a registered route and responder, **When** a client sends a supported HTTP/1.1 request matching that route, **Then** the client receives the responder's documented status, headers, and body.
2. **Given** a server with no matching route for a supported request, **When** the request is received, **Then** the client receives a documented not-found response without stopping the server.
3. **Given** a request path matches a registered route pattern but its method does not match any route for that path, **When** the request is handled, **Then** the client receives a documented method-not-allowed response without stopping the server.
4. **Given** a malformed request line or malformed headers, **When** the server receives the request, **Then** the client receives a documented bad-request response and the server remains available for later clients.

---

### User Story 2 - Build Predictable Route Handlers (Priority: P2)

A developer can define exact routes, parameterized routes, and responder behavior with clear matching rules so request handling remains understandable as an application grows.

**Why this priority**: Developers need confidence that a request reaches the intended responder. Clear routing behavior prevents accidental endpoint shadowing and confusing handler selection.

**Independent Test**: Can be tested by registering exact and parameterized routes with distinguishable responders, sending requests that exercise each matching case, and confirming the chosen responder and captured parameters are documented and deterministic.

**Acceptance Scenarios**:

1. **Given** exact and parameterized routes could both match a request, **When** the request is handled, **Then** the exact route is selected.
2. **Given** multiple parameterized routes could match a request, **When** the request is handled, **Then** the most specific documented match is selected consistently.
3. **Given** a parameterized route captures part of a request path, **When** the responder runs, **Then** the captured parameter name and value are available to the responder exactly as documented.

---

### User Story 3 - Handle Request Bodies and Responses Safely (Priority: P3)

A developer can accept documented request body forms and return static, file-backed, or custom responses without unbounded memory use or unclear connection handling.

**Why this priority**: Body handling and response writing are where server libraries commonly surprise users. The revamp should make the supported behavior safe and explicit without expanding into every HTTP/1.1 feature.

**Independent Test**: Can be tested by sending requests with supported body framing and by returning responses with and without bodies, then confirming size limits, body delivery, and connection-close or keep-alive decisions follow the documentation.

**Acceptance Scenarios**:

1. **Given** a request body uses a documented supported framing method, **When** the responder reads the body, **Then** it receives the expected bytes without the server buffering the entire body unnecessarily.
2. **Given** a request declares an unsupported transfer coding, **When** the server receives it, **Then** the client receives a documented client-error response and the responder is not invoked.
3. **Given** a responder returns a response with a body, **When** the response is written, **Then** the client receives the complete body and headers needed to interpret the response within the supported scope.

---

### User Story 4 - Diagnose and Maintain the Web Crate (Priority: P4)

A developer or maintainer can understand the supported HTTP server scope from documentation, see actionable errors when something fails, and verify behavior with repeatable tests.

**Why this priority**: The crate is not feature complete, so documentation and tests must make the boundary obvious. Maintainers need a protected baseline before later features are added deliberately.

**Independent Test**: Can be tested by following the published example and running the documented verification suite from a clean checkout without manual request crafting beyond automated tests.

**Acceptance Scenarios**:

1. **Given** a developer new to the crate, **When** they read the crate documentation, **Then** they can identify the supported route, request, response, body, and responder behavior plus the explicitly unsupported HTTP/1.1 areas.
2. **Given** a maintainer changes request parsing, routing, response writing, or responder behavior, **When** they run the documented tests, **Then** the tests cover success and failure paths for the supported server scope.
3. **Given** an operation fails during bind, accept, request parsing, validation, or response writing, **When** the failure is surfaced to the developer, **Then** it includes the failure category and enough context to decide whether the request, route, responder, or server setup caused it.

### Edge Cases

- Empty connection or end-of-stream before a request line is complete.
- Request line exceeds the documented request-line limit.
- Header section exceeds the documented header-size limit.
- Header line is missing the required separator.
- Duplicate header names are received with different casing and must be combined under one case-insensitive name, except body-framing headers governed by stricter validation rules.
- Request uses an HTTP version other than `HTTP/1.1`, an unsupported method for an otherwise matching route, or an unsupported transfer coding.
- Request provides an invalid or conflicting body length description.
- Request includes both `Content-Length` and `Transfer-Encoding` headers.
- Request asks to close the connection after the response.
- Request path matches a registered route pattern, but the method does not match any route for that path.
- Keep-alive connection receives multiple sequential supported requests from the same client.
- Route path is registered with or without a leading slash.
- Exact and parameterized routes both match the same request path.
- Parameterized terminal route receives more path segments than the route pattern contains.
- Static file responder receives a path outside its mounted directory, a directory path, a missing file, or a symbolic link target.
- Responder validation rejects a request before response construction.
- Response body reader fails while the response is being written.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The crate MUST provide a documented way for developers to create a server bound to a configured IPv4 address and port. Acceptance: successful and failed bind attempts produce documented outcomes.
- **FR-002**: The crate MUST allow developers to register routes with associated responders before the server starts accepting requests. Acceptance: a registered route can be exercised by an automated test, an unregistered route produces the documented not-found response, and a path match with no matching method produces the documented method-not-allowed response.
- **FR-003**: The crate MUST normalize route registration so paths with and without a leading slash behave consistently. Acceptance: equivalent route declarations match the same request path.
- **FR-004**: The crate MUST document and enforce deterministic route selection for exact routes, parameterized routes, and terminal parameterized routes, including method-not-allowed detection when a request path matches a registered route pattern but the method does not. Acceptance: each matching rule has an automated test with distinguishable responders, and method mismatch is tested separately from not-found routing.
- **FR-005**: The crate MUST expose captured route parameters to responders using documented parameter names and values. Acceptance: a responder can observe parameters from non-terminal and terminal route captures.
- **FR-006**: The crate MUST parse supported `HTTP/1.1` request lines, request headers, and request bodies within documented size limits. Acceptance: valid `HTTP/1.1` requests within limits are accepted and limit violations return documented client-error outcomes.
- **FR-007**: The crate MUST reject malformed request lines, malformed headers, HTTP versions other than `HTTP/1.1`, unsupported body framing, invalid body lengths, conflicting `Content-Length` plus `Transfer-Encoding` headers, and unsupported transfer codings before invoking application responders. Acceptance: each rejection path has a documented status outcome and automated test, including `505 HTTP Version Not Supported` for non-`HTTP/1.1` requests and `400 Bad Request` for conflicting `Content-Length` plus `Transfer-Encoding` headers.
- **FR-008**: The crate MUST combine duplicate request header values with commas under one case-insensitive header name for responder lookup, except body-framing headers governed by stricter validation rules. Acceptance: duplicate headers with different casing produce the documented comma-combined value visible to responders, while body-framing duplicates or conflicts are validated through the documented request rejection rules.
- **FR-009**: The crate MUST support request bodies framed by `Content-Length` and request bodies whose final transfer coding is `chunked`; all other body framing methods MUST be rejected without claiming complete HTTP/1.1 body-framing support. Acceptance: both supported framing methods work in automated tests, while unsupported framing is rejected clearly.
- **FR-010**: The crate MUST write responses with a documented status line, headers, optional body, and connection behavior. Acceptance: clients can verify response status, headers, body bytes, and whether the connection remains reusable or closes.
- **FR-011**: The crate MUST provide responder interfaces for validation and response construction that can return successful responses or documented fallback statuses. Acceptance: validation success, validation failure, response construction success, and response construction failure are each testable.
- **FR-012**: The crate MUST include documented built-in responders for static messages, static files, browser preflight responses, and single-page application fallbacks within their existing supported scope. Acceptance: each built-in responder has examples or tests covering its success path and relevant failure path.
- **FR-013**: Static file serving MUST prevent access outside the configured mount point and MUST document how missing files, directories, index files, symbolic links, and write attempts are handled. Acceptance: each file-path edge case has a deterministic status outcome.
- **FR-014**: Server, request, response, route, responder, and built-in responder failures MUST be represented as typed, actionable failures for developers. Acceptance: failures can be matched by category and their displayed messages identify what failed.
- **FR-015**: The crate documentation MUST clearly state the supported HTTP/1.1 subset and explicitly state that unsupported HTTP/1.1 features are outside this revamp. Acceptance: documentation names the supported behaviors and lists notable exclusions without promising feature completeness.
- **FR-016**: The revamp MUST avoid adding new HTTP/1.1 capabilities solely to fill protocol gaps unless they are required to stabilize already-supported behavior. Acceptance: planning can trace every behavior to an existing crate capability or to reliability, documentation, testing, or ergonomics of that capability.
- **FR-017**: Automated tests MUST cover request parsing, header handling, body handling, routing, route parameters, responder validation, response writing, keep-alive or close behavior, built-in responders, server bind failures, and malformed input within the supported scope. Acceptance: each documented behavior maps to at least one unit or integration test.
- **FR-018**: The documented example server MUST demonstrate the primary supported path without relying on stale APIs, ignored tests, or manual-only verification. Acceptance: the example can be followed from a clean checkout and its documented behavior is covered by automated tests.
- **FR-019**: The crate MUST remain available through the existing toolkit facade expectations. Acceptance: developers using the toolkit-level web module can reach the documented web crate types and responders.
- **FR-020**: Public behavior changes, including any intentional breaking changes, MUST be documented before release. Acceptance: release-facing documentation identifies the changed behavior and migration guidance for existing users.

### Key Entities *(include if feature involves data)*

- **Server**: Represents a bound HTTP server that accepts client connections and dispatches supported requests to registered routes.
- **Route**: Represents a method and path pattern, including whether the path contains parameters and how it competes with other routes.
- **Route Map**: Represents the collection of developer-registered routes and their responders.
- **Request**: Represents a parsed client request, including method, target path, version, headers, optional body, size accounting, and connection preferences within the supported scope.
- **Response**: Represents the status, headers, optional body, and connection preference sent back to a client.
- **Responder**: Represents developer-provided or built-in request handling behavior, including validation and response construction.
- **Route Parameter**: Represents a captured value from a parameterized route path that is supplied to a responder.
- **Failure**: Represents a typed server, request, response, route, or responder problem surfaced to the developer or translated into a documented client response.
- **Built-In Responder**: Represents included responder behavior for static messages, files, browser preflight requests, and single-page application fallback responses.
- **Supported HTTP Scope**: Represents the documented subset of HTTP/1.1 behavior the crate commits to support during this revamp.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer new to the crate can create a small server with one static route and one file-backed route in under 20 minutes using only the published documentation and example.
- **SC-002**: 100% of documented supported behaviors have automated tests covering the expected success path and relevant failure paths before the revamp is considered complete.
- **SC-003**: During acceptance review, every documented client-visible failure returns the expected response status and every developer-visible failure identifies the failure category without requiring internal source inspection.
- **SC-004**: A keep-alive client can complete at least two sequential supported requests on one connection with correct responses, and a close-requesting client observes the documented connection close behavior.
- **SC-005**: A single supported request with a body at the documented maximum accepted size can be processed without buffering the entire body in memory at once.
- **SC-006**: Route matching remains deterministic across at least 25 registered routes with a mix of exact and parameterized patterns.
- **SC-007**: The crate documentation and example contain zero references to stale or ignored verification paths for the supported server workflow.
- **SC-008**: The revamp leaves the web crate with no undocumented public behavior inside the declared supported HTTP server scope.

## Assumptions

- Primary users are developers building small application servers with the Webe toolkit.
- The revamp focuses on reliability, documentation, typed failures, tests, and ergonomics for the crate's current server-library scope.
- The crate uses HTTP/1.1 as its reference point, but this revamp does not attempt full HTTP/1.1 compliance or feature completion.
- Unsupported protocol areas remain outside scope unless needed to make already-supported request parsing, routing, response writing, or responder behavior correct and testable.
- Compatibility with existing users matters, but documented breaking changes are acceptable when they make the server behavior clearer, safer, or testable.
- The basic server example remains the primary developer-facing demonstration of the crate's supported workflow.
- Security-sensitive behavior for file serving is part of the supported scope because static file responders already exist.
