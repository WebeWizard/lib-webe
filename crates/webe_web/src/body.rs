//! Request and response body-framing decisions.
//!
//! This module centralizes the supported HTTP/1.1 body framings and their
//! rejections so the rules live in one validated place:
//!
//! - **Request**: a single `Content-Length`, or a `Transfer-Encoding` whose final
//!   coding is `chunked`, or no body. Both headers present, an unparseable
//!   `Content-Length`, or an unsupported transfer coding are rejected.
//! - **Response**: a known length sends `Content-Length`; a streamed body of
//!   unknown length sends `Transfer-Encoding: chunked`; a bodyless response sends
//!   neither.

use std::collections::HashMap;

/// Why a body could not be framed within the supported subset. Maps to `400`.
#[derive(Debug, PartialEq, Eq)]
pub enum BodyError {
    /// Both `Content-Length` and `Transfer-Encoding` were present.
    ConflictingFraming,
    /// A `Content-Length` value could not be parsed as a byte count.
    UnparseableLength,
    /// The final transfer coding was not `chunked`.
    UnsupportedCoding,
}

impl std::fmt::Display for BodyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BodyError::ConflictingFraming => write!(
                f,
                "body: both Content-Length and Transfer-Encoding present; send only one (400)"
            ),
            BodyError::UnparseableLength => {
                write!(f, "body: Content-Length is not a valid byte count (400)")
            }
            BodyError::UnsupportedCoding => write!(
                f,
                "body: unsupported transfer coding; only a final 'chunked' coding is accepted (400)"
            ),
        }
    }
}

/// How an incoming request body is framed.
#[derive(Debug, PartialEq, Eq)]
pub enum RequestBody {
    /// No request body.
    None,
    /// A body of exactly this many bytes (`Content-Length`).
    Length(u64),
    /// A `chunked` transfer-coded body.
    Chunked,
}

/// Decides the request body framing from the (already lowercased) request headers.
///
/// Returns a [`BodyError`] for conflicting framing headers, an unparseable
/// `Content-Length`, or a transfer coding whose final coding is not `chunked`.
pub fn decide_request_body(
    headers: Option<&HashMap<String, String>>,
) -> Result<RequestBody, BodyError> {
    let content_length = headers.and_then(|h| h.get("content-length"));
    let transfer_encoding = headers.and_then(|h| h.get("transfer-encoding"));

    match (content_length, transfer_encoding) {
        // Conflicting framing is a request-smuggling risk; reject outright.
        (Some(_), Some(_)) => Err(BodyError::ConflictingFraming),
        (Some(length), None) => {
            // A duplicate Content-Length is comma-combined upstream (e.g. "5,5")
            // and therefore fails to parse here, which is the desired rejection.
            let bytes = length
                .trim()
                .parse::<u64>()
                .map_err(|_| BodyError::UnparseableLength)?;
            Ok(RequestBody::Length(bytes))
        }
        (None, Some(encoding)) => {
            let codings: Vec<&str> = encoding.split(',').map(|c| c.trim()).collect();
            let final_coding = codings.last().copied().unwrap_or("");
            if final_coding.eq_ignore_ascii_case("chunked") {
                Ok(RequestBody::Chunked)
            } else {
                Err(BodyError::UnsupportedCoding)
            }
        }
        (None, None) => Ok(RequestBody::None),
    }
}

/// How an outgoing response body is framed.
#[derive(Debug, PartialEq, Eq)]
pub enum ResponseFraming {
    /// No body; send neither `Content-Length` nor `Transfer-Encoding`.
    None,
    /// Known length; the `Content-Length` header is already present.
    Length,
    /// Unknown length; stream with `Transfer-Encoding: chunked`.
    Chunked,
}

/// Decides response framing: a bodyless response is [`ResponseFraming::None`]; a
/// body with a `Content-Length` header is [`ResponseFraming::Length`]; any other
/// body is streamed as [`ResponseFraming::Chunked`].
pub fn decide_response_framing(
    has_body: bool,
    headers: &HashMap<String, String>,
) -> ResponseFraming {
    if !has_body {
        return ResponseFraming::None;
    }
    if headers
        .keys()
        .any(|k| k.eq_ignore_ascii_case("content-length"))
    {
        ResponseFraming::Length
    } else {
        ResponseFraming::Chunked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn no_framing_headers_means_no_body() {
        let h = headers(&[]);
        assert_eq!(decide_request_body(Some(&h)), Ok(RequestBody::None));
        assert_eq!(decide_request_body(None), Ok(RequestBody::None));
    }

    #[test]
    fn single_content_length_is_parsed() {
        let h = headers(&[("content-length", "42")]);
        assert_eq!(decide_request_body(Some(&h)), Ok(RequestBody::Length(42)));
    }

    #[test]
    fn unparseable_content_length_is_rejected() {
        let h = headers(&[("content-length", "abc")]);
        assert_eq!(
            decide_request_body(Some(&h)),
            Err(BodyError::UnparseableLength)
        );
        // a comma-combined duplicate also fails to parse
        let dup = headers(&[("content-length", "5,5")]);
        assert_eq!(
            decide_request_body(Some(&dup)),
            Err(BodyError::UnparseableLength)
        );
    }

    #[test]
    fn final_chunked_coding_is_accepted() {
        let h = headers(&[("transfer-encoding", "gzip, chunked")]);
        assert_eq!(decide_request_body(Some(&h)), Ok(RequestBody::Chunked));
    }

    #[test]
    fn non_chunked_final_coding_is_rejected() {
        let h = headers(&[("transfer-encoding", "gzip")]);
        assert_eq!(
            decide_request_body(Some(&h)),
            Err(BodyError::UnsupportedCoding)
        );
    }

    #[test]
    fn both_framing_headers_conflict() {
        let h = headers(&[("content-length", "5"), ("transfer-encoding", "chunked")]);
        assert_eq!(
            decide_request_body(Some(&h)),
            Err(BodyError::ConflictingFraming)
        );
    }

    #[test]
    fn response_framing_selection() {
        assert_eq!(
            decide_response_framing(false, &headers(&[])),
            ResponseFraming::None
        );
        assert_eq!(
            decide_response_framing(true, &headers(&[("Content-Length", "5")])),
            ResponseFraming::Length
        );
        assert_eq!(
            decide_response_framing(true, &headers(&[])),
            ResponseFraming::Chunked
        );
    }
}
