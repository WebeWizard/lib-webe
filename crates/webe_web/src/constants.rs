//! Shared constants: buffer sizes, request size limits, and MIME type tables.

/// Default read/write buffer size (8 KB).
pub const WEBE_BUFFER_SIZE: usize = 8192; // 8KB , default rust buffer size

/// Number of seconds in 30 days, used for cookie/session lifetimes.
pub const SECONDS_30_DAYS: u32 = 2592000; // 30 days

// ---STRUCT/DATA SIZES---
// Requests
/// Maximum size, in bytes, of a single request line before it is rejected.
pub const MAX_REQUEST_LINE_SIZE: usize = 10480000; // 1MB
/// Maximum combined size, in bytes, of all request headers before rejection.
pub const MAX_HEADERS_SIZE: usize = 2048000; // 2MB - Maximum size of all headers combined
/// Maximum overall request size, in bytes.
pub const MAX_REQUEST_SIZE: usize = 51200000; // 50MB

// ---MIME TYPES---
// TODO: Currently assuming everything is utf-8 encoded
/// MIME mapping for `.js` files.
pub const MIME_JS: (&str, &str) = ("js", "application/javascript; charset=utf-8");
/// MIME mapping for `.json` files.
pub const MIME_JSON: (&str, &str) = ("json", "application/json; charset=utf-8");
/// MIME mapping for `.htm` files.
pub const MIME_HTM: (&str, &str) = ("htm", "text/html; charset=utf-8");
/// MIME mapping for `.html` files.
pub const MIME_HTML: (&str, &str) = ("html", "text/html; charset=utf-8");
/// MIME mapping for `.css` files.
pub const MIME_CSS: (&str, &str) = ("css", "text/css; charset=utf-8");
/// MIME mapping for `.gif` files.
pub const MIME_GIF: (&str, &str) = ("gif", "image/gif");
/// MIME mapping for `.jpg` files.
pub const MIME_JPG: (&str, &str) = ("jpg", "image/jpeg");
/// MIME mapping for `.jpeg` files.
pub const MIME_JPEG: (&str, &str) = ("jpeg", "image/jpeg");
/// MIME mapping for `.png` files.
pub const MIME_PNG: (&str, &str) = ("png", "image/png");
/// MIME mapping for `.svg` files.
pub const MIME_SVG: (&str, &str) = ("svg", "image/svg+xml");
/// MIME mapping for `.ico` files.
pub const MIME_ICO: (&str, &str) = ("ico", "image/x-icon");

/// Fallback MIME type for files with no known extension mapping.
pub const MIME_OCTET_STREAM: &str = "application/octet-stream";

/// The default extension-to-MIME table used by `FileResponder`.
pub const DEFAULT_MIME_TYPES: [(&str, &str); 11] = [
    MIME_JS, MIME_JSON, MIME_HTM, MIME_HTML, MIME_CSS, MIME_GIF, MIME_JPG, MIME_JPEG, MIME_PNG,
    MIME_SVG, MIME_ICO,
];
