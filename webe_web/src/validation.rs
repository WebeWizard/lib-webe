use super::status::Status;
use std::any::Any;

pub type Validation = Option<Box<dyn Any>>;

pub type ValidationResult = Result<Validation, Status>;
