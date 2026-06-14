# One Library for One Wizard

This project aims to provide a set of tools for creating fast and stable Rust application servers suitable for a hobbyist developer.

## Workspace Layout

The repository is a Rust workspace with the root crate acting as the facade library. Use the root `webe` crate when you want the whole toolkit:

- `webe::web` re-exports `webe_web`
- `webe::auth` re-exports `webe_auth` behind the `auth` feature
- `webe::log` re-exports `webe_log`
- `webe::args` re-exports `webe_args`
- `webe::id` re-exports `webe_id` behind the `id` feature

Implementation crates live under `crates/`, and runnable examples live under `examples/`.

Default builds skip `webe_auth` because it requires local MySQL client libraries. Enable it explicitly with `--features auth` when the auth stack is needed.

## HTTP Server
Using HTTP 1.1 Spec as a reference but not a requirement.

Responders make up the basic request handler.  Responders can be nested to conveniently provide authentication, logging, or any repeated custom behavior on each request.

Responders are mapped to a Route.  The servers parses the URL of each request and chooses the best matching route.
Ex. Consider a server with 2 endpoints:  
`/post`,  
`/post/<post_num>`,  

A request to `/post/123` would match route `/post/\<post_num\>` because it contains the correct number of url parts. The parameter <post_num> can then read from the responder.  
A request to `/post/123/edit` would also match `/post/<post_num>`. And the value `123/edit` would be passed to the Responder.  We could configure the responder to parse this, or we could add an additional responder at route `/post/<post_num>/<action>`.

## Authentication
 - Account management (Basic Account CRUD operations)
 - BCrypt hashed passwords
 - Token based Sessions
 - Includes prebuilt HTTP server Responders


## Unique ID Generation
Webe includes a compact, sortable 64-bit unique ID generator through the optional `id` facade feature:

```toml
[dependencies]
webe = { version = "0.1", features = ["id"] }
```

```rust
let mut generator = webe::id::Generator::builder(webe::id::NodeId::from_u8(1)).build()?;
let id = generator.generate()?;
let components = id.components();
```

Each WebeID stores 40 bits of milliseconds since a custom epoch, 8 bits of node identity, and 16 bits of per-node sequence. Default generation fails fast with typed errors when the clock rewinds, the time range is exhausted, or one node consumes all 65,536 sequence values in one millisecond.

Enable `id-tokio` when Tokio callers want the optional bounded backpressure helper:

```toml
[dependencies]
webe = { version = "0.1", features = ["id-tokio"] }
```

The direct implementation crate is `webe_id`; see `crates/webe_id/README.md` for restart-marker guidance, limits, errors, and benchmark reporting.