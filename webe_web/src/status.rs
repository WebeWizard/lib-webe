pub struct Status {
  pub code: u16,
  pub reason: String,
}

impl Status {
  // this is only for standard status codes.
  // if you want a custom status, you'll have to make it yourself
  pub fn from_standard_code(code: u16) -> Status {
    let reason = Status::get_standard_reason(code);
    return Status {
      code: code,
      reason: reason.to_owned(),
    };
  }

  // these are all status codes defined by HTTP 1.1
  // this SHOULD NOT be used as a strict list of codes because custom codes are allowed
  pub fn get_standard_reason(code: u16) -> &'static str {
    match code {
      100 => "Continue",
      101 => "Switching Protocols",
      200 => "OK",
      201 => "Created",
      202 => "Accepted",
      203 => "Non-Authoritative Information",
      204 => "No Content",
      205 => "Reset Content",
      206 => "Partial Content",
      300 => "Multiple Choices",
      301 => "Moved Permanently",
      302 => "Found",
      303 => "See Other",
      304 => "Not Modified",
      305 => "Use Proxy",
      307 => "Temporary Redirect",
      400 => "Bad Request",
      401 => "Unauthorized",
      402 => "Payment Required",
      403 => "Forbidden",
      404 => "Not Found",
      405 => "Method Not Allowed",
      406 => "Not Acceptable",
      407 => "Proxy Authentication Required",
      408 => "Request Time-out",
      409 => "Conflict",
      410 => "Gone",
      411 => "Length Required",
      412 => "Precondition Failed",
      413 => "Request Entity Too Large",
      414 => "Request-URI Too Large",
      415 => "Unsupported Media Type",
      416 => "Requested range not satisfiable",
      417 => "Expectation Failed",
      500 => "Internal Server Error",
      501 => "Not Implemented",
      502 => "Bad Gateway",
      503 => "Service Unavailable",
      504 => "Gateway Time-out",
      505 => "HTTP Version not supported",
      _ => "Non-Standard HTTP status.  No reason defined",
    }
  }
}
