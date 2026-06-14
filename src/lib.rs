/// HTTP server, responders, routing, and request/response types.
#[cfg(feature = "web")]
pub use webe_web as web;

/// Account management, session tokens, and auth HTTP responders.
#[cfg(feature = "auth")]
pub use webe_auth as auth;

/// Structured logging with pluggable sinks.
#[cfg(feature = "log")]
pub use webe_log as log;

/// Command-line argument parsing.
#[cfg(feature = "args")]
pub use webe_args as args;

/// Compact, sortable unique ID generation.
#[cfg(feature = "id")]
pub use webe_id as id;
