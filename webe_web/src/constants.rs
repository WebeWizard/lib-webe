// ---PERF?---
pub const WEBE_BUFFER_SIZE: usize = 8192; // 8KB , default rust buffer size

// ---TIME---
pub const SECONDS_30_DAYS: u32 = 2592000; // 30 days

// ---STRUCT SIZES---
pub const MAX_HEADER_SIZE: usize = 2048000; // 2MB
pub const MAX_REQUEST_SIZE: usize = 51200000; // 50MB

// ---MIME TYPES---
// TODO: Currently assuming everything is utf-8 encoded
pub const MIME_JS: (&str, &str) = ("js", "application/javascript; charset=utf-8");
pub const MIME_JSON: (&str, &str) = ("json", "application/json; charset=utf-8");
pub const MIME_HTM: (&str, &str) = ("htm", "text/html; charset=utf-8");
pub const MIME_HTML: (&str, &str) = ("html", "text/html; charset=utf-8");
pub const MIME_CSS: (&str, &str) = ("css", "text/css; charset=utf-8");
pub const MIME_GIF: (&str, &str) = ("gif", "image/gif");
pub const MIME_JPG: (&str, &str) = ("jpg", "image/jpeg");
pub const MIME_JPEG: (&str, &str) = ("jpeg", "image/jpeg");
pub const MIME_PNG: (&str, &str) = ("png", "image/png");
pub const MIME_SVG: (&str, &str) = ("svg", "image/svg+xml");
pub const MIME_ICO: (&str, &str) = ("ico", "image/x-icon");

pub const MIME_OCTET_STREAM: &str = "application/octet-stream";

pub const DEFAULT_MIME_TYPES: [(&str, &str); 11] = [
  MIME_JS, MIME_JSON, MIME_HTM, MIME_HTML, MIME_CSS, MIME_GIF, MIME_JPG, MIME_JPEG, MIME_PNG,
  MIME_SVG, MIME_ICO
];
