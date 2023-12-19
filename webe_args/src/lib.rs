use std::collections::HashMap;
use std::env;

pub struct ArgOpts {
  pub short: Option<String>,
  pub description: Option<String>,
  pub is_required: bool,
  pub is_flag: bool,
  pub validation: Option<Box<dyn Fn(&str) -> bool>>,
}

pub const DEFAULT_OPTS: ArgOpts = ArgOpts {
  short: None,
  description: None,
  is_required: true,
  is_flag: false,
  validation: None,
};

pub fn format_as_long(name: &str) -> String {
  format!("--{}", name)
}

pub fn format_as_short(name: &str) -> String {
  format!("-{}", name)
}

#[derive(Debug)]
pub enum ArgError {
  NoArgOpt,                   // There is no definition for this argument.
  ArgNotFound,                // Not found in env args
  RequiredNotFound,           // Required Arg not found in args
  ValueNotFound,              // found arg but no value
  InvalidValue,               // validation func returned false
}

pub struct Args {
  inner: HashMap<String, ArgOpts>,
}

impl Args {
  pub fn new() -> Args {
    Args {
      inner: HashMap::new(),
    }
  }

  pub fn add(&mut self, name: String, options: ArgOpts) {
    self.inner.insert(name, options);
  }

  // gets the value of an argument and validates it if necessary
  pub fn get(&self, name: &str) -> Result<Option<String>, ArgError> {
    match self.inner.get(name) {
      Some(argopt) => {
        match env::args().position(|arg: String| {
          arg == format_as_long(name)
            || match &argopt.short {
              Some(short_name) => arg == format_as_short(&short_name),
              None => false,
            }
        }) {
          Some(pos) => {
            if argopt.is_flag {
              return Ok(None);
            } else {
              match env::args().nth(pos + 1) {
                Some(val) => match &argopt.validation {
                  Some(validation_func) => {
                    if validation_func(val.as_str()) {
                      return Ok(Some(val));
                    } else {
                      return Err(ArgError::InvalidValue);
                    }
                  }
                  None => return Ok(Some(val)),
                },
                None => return Err(ArgError::ValueNotFound),
              }
            }
          },
          None => {
            if argopt.is_required {
              return Err(ArgError::RequiredNotFound);
            }
              else { return Ok(None); }
          }
        }
      }
      None => return Err(ArgError::NoArgOpt),
    }
  }

  // checks that all args are present and valid, otherwise panics with friendly message
  pub fn parse_args(&self) {
    for (name, _arg) in self.inner.iter() {
      if let Err(err) = self.get(name) {
        match err {
          ArgError::RequiredNotFound => panic!("Missing required argument: {}", name),
          ArgError::ArgNotFound => {}, // arg wasn't found, but it wasn't required anyways, so move along.
          ArgError::InvalidValue => panic!("An invalid value was provided for argument: {}", name),
          ArgError::ValueNotFound => panic!("Expected a value for argument: {}", name),
          ArgError::NoArgOpt => panic!("This error shouldn't be possible here"),
        }
      }
    }
  }
}
