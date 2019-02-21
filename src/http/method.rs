use super::request::Request;
use super::response::Response;

trait Method {
    fn validate_request(request: &Request) -> bool;
    fn build_response(request: &Request) -> Response;
}