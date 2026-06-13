use super::status::Status;
use std::any::Any;

pub type Validation = Option<Box<dyn Any + Send>>;

pub type ValidationResult = Result<Validation, Status>;
