use super::request::Request;
use super::response::Response;

pub enum ProcessError {
    NotValid
}

// takes a request and creates a response
trait Processor {
    fn process(request: Request) -> Result<Response, ProcessError>;
}