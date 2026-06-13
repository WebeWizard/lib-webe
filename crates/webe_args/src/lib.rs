use std::collections::HashMap;
use std::env;

pub type ArgValidation = Box<dyn Fn(&str) -> bool>;

pub struct ArgOpts {
    pub short: Option<String>,
    pub description: Option<String>,
    pub is_required: bool,
    pub is_flag: bool,
    pub validation: Option<ArgValidation>,
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
    NoArgOpt,         // There is no definition for this argument.
    ArgNotFound,      // Not found in env args
    RequiredNotFound, // Required Arg not found in args
    ValueNotFound,    // found arg but no value
    InvalidValue,     // validation func returned false
}

pub struct Args {
    inner: HashMap<String, ArgOpts>,
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
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
        let Some(argopt) = self.inner.get(name) else {
            return Err(ArgError::NoArgOpt);
        };

        match env::args().position(|arg: String| {
            arg == format_as_long(name)
                || argopt
                    .short
                    .as_ref()
                    .is_some_and(|short_name| arg == format_as_short(short_name))
        }) {
            Some(_) if argopt.is_flag => Ok(None),
            Some(pos) => {
                let Some(val) = env::args().nth(pos + 1) else {
                    return Err(ArgError::ValueNotFound);
                };

                match &argopt.validation {
                    Some(validation_func) if !validation_func(val.as_str()) => {
                        Err(ArgError::InvalidValue)
                    }
                    _ => Ok(Some(val)),
                }
            }
            None if argopt.is_required => Err(ArgError::RequiredNotFound),
            None => Ok(None),
        }
    }

    // checks that all args are present and valid, otherwise panics with friendly message
    pub fn parse_args(&self) {
        for (name, _arg) in self.inner.iter() {
            if let Err(err) = self.get(name) {
                match err {
                    ArgError::RequiredNotFound => panic!("Missing required argument: {}", name),
                    ArgError::ArgNotFound => {} // arg wasn't found, but it wasn't required anyways, so move along.
                    ArgError::InvalidValue => {
                        panic!("An invalid value was provided for argument: {}", name)
                    }
                    ArgError::ValueNotFound => panic!("Expected a value for argument: {}", name),
                    ArgError::NoArgOpt => panic!("This error shouldn't be possible here"),
                }
            }
        }
    }
}
