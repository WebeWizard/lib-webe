#![forbid(unsafe_code)]
#![warn(missing_docs)]
//! Parse simple command-line options without external dependencies.
//!
//! `webe_args` lets you declare named options (a long name, an optional short
//! alias, a description, required/optional status, flag/value behavior, and an
//! optional value-validation rule), then either read a single option or validate
//! a whole command line in one pass. Core entry points accept a caller-supplied
//! token slice (`&[String]`) so behavior is deterministic and easy to test
//! without touching the process environment.
//!
//! # Quick start
//!
//! ```
//! use webe_args::{OptionDef, OptionResult, Registry};
//!
//! let mut registry = Registry::new();
//! registry
//!     .add(OptionDef::value("port").short("p").required().validate(|v| v.parse::<u16>().is_ok()))
//!     .add(OptionDef::flag("verbose").short("v"));
//!
//! let tokens = vec!["--port".to_string(), "8080".to_string(), "-v".to_string()];
//!
//! // Validate the whole command line at startup.
//! let report = registry.validate(&tokens);
//! assert!(report.is_success());
//!
//! // Read individual options.
//! assert_eq!(registry.read("port", &tokens).unwrap(), OptionResult::Value("8080".to_string()));
//! assert_eq!(registry.read("verbose", &tokens).unwrap(), OptionResult::Flag);
//! ```
//!
//! # Token interpretation
//!
//! * A token equal to a declared `--long_name` or `-short_alias` is an option
//!   token.
//! * Any token beginning with `-` is treated as an option token, never as the
//!   value of a preceding value option.
//! * A bare (non-`-`) token immediately following a value option is that option's
//!   value, returned verbatim.
//! * Any other bare token is an unexpected positional argument.
//!
//! # Breaking changes
//!
//! This release replaces the previous `Args` / `ArgOpts` / `parse_args` design.
//! The old API read `std::env::args()` directly and `panic!`ed on bad input. The
//! new API takes caller-supplied tokens and returns typed [`ParseFailure`] values
//! via [`Registry::read`] and [`Registry::validate`]. See the crate README for a
//! migration guide.

use std::collections::{HashMap, HashSet};
use std::fmt;

/// A value-validation predicate applied to a value option's supplied value.
///
/// The predicate returns `true` when the value is acceptable and `false` when it
/// should be rejected with [`ParseFailure::InvalidValue`].
pub type Validator = Box<dyn Fn(&str) -> bool>;

/// One accepted command-line option.
///
/// Build an `OptionDef` with [`OptionDef::value`] (a value-taking option) or
/// [`OptionDef::flag`] (a presence-only flag), then chain the builder methods to
/// attach a short alias, description, required status, or validation rule.
///
/// # Examples
///
/// ```
/// use webe_args::OptionDef;
///
/// // A required value option with a short alias and validation.
/// let port = OptionDef::value("port")
///     .short("p")
///     .description("TCP port to bind")
///     .required()
///     .validate(|v| v.parse::<u16>().is_ok());
///
/// // An optional boolean flag.
/// let verbose = OptionDef::flag("verbose").short("v");
/// ```
pub struct OptionDef {
    long_name: String,
    short_alias: Option<String>,
    description: Option<String>,
    is_required: bool,
    is_flag: bool,
    validation: Option<Validator>,
}

impl OptionDef {
    /// Begin defining a value-taking option identified by `long_name`.
    ///
    /// The option is optional and unvalidated by default. The `long_name` is used
    /// on the command line as `--long_name`.
    ///
    /// # Examples
    ///
    /// ```
    /// use webe_args::OptionDef;
    /// let name = OptionDef::value("name");
    /// ```
    pub fn value(long_name: impl Into<String>) -> Self {
        OptionDef {
            long_name: long_name.into(),
            short_alias: None,
            description: None,
            is_required: false,
            is_flag: false,
            validation: None,
        }
    }

    /// Begin defining a flag (presence-only) option identified by `long_name`.
    ///
    /// A flag never takes a value and never consumes a following token. The
    /// `long_name` is used on the command line as `--long_name`.
    ///
    /// # Examples
    ///
    /// ```
    /// use webe_args::OptionDef;
    /// let verbose = OptionDef::flag("verbose");
    /// ```
    pub fn flag(long_name: impl Into<String>) -> Self {
        OptionDef {
            long_name: long_name.into(),
            short_alias: None,
            description: None,
            is_required: false,
            is_flag: true,
            validation: None,
        }
    }

    /// Attach a short alias, used on the command line as `-alias`.
    ///
    /// # Examples
    ///
    /// ```
    /// use webe_args::OptionDef;
    /// let port = OptionDef::value("port").short("p");
    /// ```
    pub fn short(mut self, alias: impl Into<String>) -> Self {
        self.short_alias = Some(alias.into());
        self
    }

    /// Attach human-readable help text describing the option.
    ///
    /// # Examples
    ///
    /// ```
    /// use webe_args::OptionDef;
    /// let port = OptionDef::value("port").description("TCP port to bind");
    /// ```
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Mark the option as required, so its absence is a
    /// [`ParseFailure::MissingRequired`].
    ///
    /// # Examples
    ///
    /// ```
    /// use webe_args::OptionDef;
    /// let port = OptionDef::value("port").required();
    /// ```
    pub fn required(mut self) -> Self {
        self.is_required = true;
        self
    }

    /// Attach a value-validation predicate.
    ///
    /// The predicate runs against a value option's supplied value; returning
    /// `false` yields a [`ParseFailure::InvalidValue`]. Validation never applies
    /// to flags.
    ///
    /// # Examples
    ///
    /// ```
    /// use webe_args::OptionDef;
    /// let port = OptionDef::value("port").validate(|v| v.parse::<u16>().is_ok());
    /// ```
    pub fn validate(mut self, predicate: impl Fn(&str) -> bool + 'static) -> Self {
        self.validation = Some(Box::new(predicate));
        self
    }

    /// The option's long name (without the leading `--`).
    pub fn long_name(&self) -> &str {
        &self.long_name
    }

    /// The option's short alias (without the leading `-`), if any.
    pub fn short_alias(&self) -> Option<&str> {
        self.short_alias.as_deref()
    }

    /// The option's description, if any.
    pub fn description_text(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Whether the option must be present.
    pub fn is_required(&self) -> bool {
        self.is_required
    }

    /// Whether the option is a presence-only flag.
    pub fn is_flag(&self) -> bool {
        self.is_flag
    }
}

/// The outcome of reading one declared option from supplied tokens.
///
/// Returned by [`Registry::read`] on success; failures are reported as a
/// [`ParseFailure`] via the `Err` arm of that method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionResult {
    /// A value option was present with the contained value (verbatim).
    Value(String),
    /// A flag option was present.
    Flag,
    /// An optional option was absent (a successful no-value result).
    Absent,
}

/// A distinct, developer-actionable problem discovered while reading or
/// validating options.
///
/// Each variant names the affected option (by long name) or the offending token,
/// so messages are actionable without inspecting parser internals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseFailure {
    /// A required option was absent. Carries the option long name.
    MissingRequired(String),
    /// A value option had no value (end of input, or the next token began with
    /// `-`). Carries the option long name.
    MissingValue(String),
    /// A value option's value failed its validation rule. Carries the option long
    /// name.
    InvalidValue(String),
    /// An option token had no declared long name or short alias. Carries the
    /// offending token.
    UnknownOption(String),
    /// The same option was supplied more than once. Carries the option long name.
    DuplicateOption(String),
    /// A bare positional token was not consumed as a value. Carries the offending
    /// token.
    UnexpectedArgument(String),
    /// Two definitions shared a long name or short alias. Carries the conflicting
    /// name or alias.
    ConflictingDefinition(String),
    /// A read requested an option that was never defined. Carries the requested
    /// name. This indicates a programming error rather than bad user input.
    UndeclaredLookup(String),
}

impl fmt::Display for ParseFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseFailure::MissingRequired(name) => {
                write!(f, "missing required option '--{name}'")
            }
            ParseFailure::MissingValue(name) => {
                write!(
                    f,
                    "option '--{name}' requires a value but none was provided"
                )
            }
            ParseFailure::InvalidValue(name) => {
                write!(f, "option '--{name}' was given an invalid value")
            }
            ParseFailure::UnknownOption(token) => {
                write!(f, "unknown option '{token}'")
            }
            ParseFailure::DuplicateOption(name) => {
                write!(f, "option '{name}' was supplied more than once")
            }
            ParseFailure::UnexpectedArgument(token) => {
                write!(f, "unexpected argument '{token}'")
            }
            ParseFailure::ConflictingDefinition(name) => {
                write!(f, "conflicting option definition for '{name}'")
            }
            ParseFailure::UndeclaredLookup(name) => {
                write!(f, "attempted to read undeclared option '{name}'")
            }
        }
    }
}

impl std::error::Error for ParseFailure {}

/// The result of validating a full command line.
///
/// A report is either successful or carries every discovered failure in
/// deterministic order: [`ParseFailure::ConflictingDefinition`] first, then
/// input-derived failures by left-to-right token position, then
/// [`ParseFailure::MissingRequired`] by option registration order.
///
/// # Examples
///
/// ```
/// use webe_args::{OptionDef, Registry};
///
/// let mut registry = Registry::new();
/// registry.add(OptionDef::value("port").required());
///
/// let report = registry.validate(&["--bogus".to_string()]);
/// assert!(!report.is_success());
/// assert_eq!(report.failures().len(), 2); // UnknownOption + MissingRequired
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    failures: Vec<ParseFailure>,
}

impl ValidationReport {
    /// Whether validation succeeded (no failures were discovered).
    pub fn is_success(&self) -> bool {
        self.failures.is_empty()
    }

    /// The discovered failures in deterministic order. Empty on success.
    pub fn failures(&self) -> &[ParseFailure] {
        &self.failures
    }
}

/// A registry of option definitions, preserving registration order for
/// deterministic reporting.
///
/// Add definitions with [`Registry::add`], then read a single option with
/// [`Registry::read`] or validate a whole command line with
/// [`Registry::validate`].
///
/// # Examples
///
/// ```
/// use webe_args::{OptionDef, Registry};
///
/// let mut registry = Registry::new();
/// registry
///     .add(OptionDef::value("port").required())
///     .add(OptionDef::flag("verbose"));
/// ```
#[derive(Default)]
pub struct Registry {
    defs: Vec<OptionDef>,
}

impl Registry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Registry { defs: Vec::new() }
    }

    /// Register an option definition, preserving registration order.
    ///
    /// Returns `&mut Self` so calls can be chained. Conflicting definitions
    /// (duplicate long names or short aliases) are not rejected here; they are
    /// reported as [`ParseFailure::ConflictingDefinition`] by
    /// [`Registry::validate`].
    pub fn add(&mut self, definition: OptionDef) -> &mut Self {
        self.defs.push(definition);
        self
    }

    /// Read a single declared option from `tokens`.
    ///
    /// Returns:
    /// * `Ok(OptionResult::Value(v))` for a present value option (value verbatim),
    /// * `Ok(OptionResult::Flag)` for a present flag,
    /// * `Ok(OptionResult::Absent)` for an absent optional option,
    /// * `Err(ParseFailure::MissingRequired)` for an absent required option,
    /// * `Err(ParseFailure::MissingValue)` for a value option with no value,
    /// * `Err(ParseFailure::InvalidValue)` when validation rejects the value,
    /// * `Err(ParseFailure::UndeclaredLookup)` when `name` was never defined.
    ///
    /// The first occurrence of the option (long or short form) is used.
    ///
    /// # Examples
    ///
    /// ```
    /// use webe_args::{OptionDef, OptionResult, Registry};
    ///
    /// let mut registry = Registry::new();
    /// registry.add(OptionDef::value("name"));
    ///
    /// let tokens = vec!["--name".to_string(), "webe is great".to_string()];
    /// assert_eq!(
    ///     registry.read("name", &tokens).unwrap(),
    ///     OptionResult::Value("webe is great".to_string())
    /// );
    /// ```
    ///
    /// Reading an option that was never defined is a programming error:
    ///
    /// ```
    /// use webe_args::{ParseFailure, Registry};
    ///
    /// let registry = Registry::new();
    /// assert_eq!(
    ///     registry.read("nope", &[]),
    ///     Err(ParseFailure::UndeclaredLookup("nope".to_string()))
    /// );
    /// ```
    pub fn read(&self, name: &str, tokens: &[String]) -> Result<OptionResult, ParseFailure> {
        let Some(def) = self.defs.iter().find(|d| d.long_name == name) else {
            return Err(ParseFailure::UndeclaredLookup(name.to_string()));
        };

        let long = format!("--{}", def.long_name);
        let short = def.short_alias.as_ref().map(|s| format!("-{s}"));

        let position = tokens
            .iter()
            .position(|tok| *tok == long || short.as_deref() == Some(tok.as_str()));

        match position {
            None if def.is_required => Err(ParseFailure::MissingRequired(def.long_name.clone())),
            None => Ok(OptionResult::Absent),
            Some(_) if def.is_flag => Ok(OptionResult::Flag),
            Some(pos) => match tokens.get(pos + 1) {
                Some(value) if !value.starts_with('-') => {
                    if value_is_invalid(def, value) {
                        return Err(ParseFailure::InvalidValue(def.long_name.clone()));
                    }
                    Ok(OptionResult::Value(value.clone()))
                }
                _ => Err(ParseFailure::MissingValue(def.long_name.clone())),
            },
        }
    }

    /// Validate a full command line in a single pass, collecting every failure.
    ///
    /// The returned [`ValidationReport`] is successful only when the definitions
    /// are conflict-free and every token is accounted for. Failures are ordered
    /// deterministically: conflicting definitions first, then input failures by
    /// left-to-right token position, then missing-required failures by
    /// registration order.
    ///
    /// # Examples
    ///
    /// Success:
    ///
    /// ```
    /// use webe_args::{OptionDef, Registry};
    ///
    /// let mut registry = Registry::new();
    /// registry.add(OptionDef::value("port").required());
    /// assert!(registry.validate(&["--port".to_string(), "8080".to_string()]).is_success());
    /// ```
    ///
    /// All failures in order:
    ///
    /// ```
    /// use webe_args::{OptionDef, ParseFailure, Registry};
    ///
    /// let mut registry = Registry::new();
    /// registry.add(OptionDef::value("port").required());
    ///
    /// let tokens = vec!["--bogus".to_string(), "extra".to_string(), "--port".to_string()];
    /// let report = registry.validate(&tokens);
    /// assert_eq!(
    ///     report.failures(),
    ///     &[
    ///         ParseFailure::UnknownOption("--bogus".to_string()),
    ///         ParseFailure::UnexpectedArgument("extra".to_string()),
    ///         ParseFailure::MissingValue("port".to_string()),
    ///     ]
    /// );
    /// ```
    pub fn validate(&self, tokens: &[String]) -> ValidationReport {
        let mut failures = Vec::new();

        // 1. Definition-level conflicts, in registration order.
        let mut seen_long: HashSet<&str> = HashSet::new();
        let mut seen_short: HashSet<&str> = HashSet::new();
        for def in &self.defs {
            if !seen_long.insert(def.long_name.as_str()) {
                failures.push(ParseFailure::ConflictingDefinition(def.long_name.clone()));
            }
            match &def.short_alias {
                Some(short) if !seen_short.insert(short.as_str()) => {
                    failures.push(ParseFailure::ConflictingDefinition(short.clone()));
                }
                _ => {}
            }
        }

        // Build a token-string -> definition-index lookup once (linear).
        let mut lookup: HashMap<String, usize> = HashMap::new();
        for (idx, def) in self.defs.iter().enumerate() {
            lookup.entry(format!("--{}", def.long_name)).or_insert(idx);
            if let Some(short) = &def.short_alias {
                lookup.entry(format!("-{short}")).or_insert(idx);
            }
        }

        // 2. Input-derived failures, by left-to-right token position.
        let mut present: HashSet<usize> = HashSet::new();
        let mut i = 0;
        while i < tokens.len() {
            let token = &tokens[i];
            if let Some(&idx) = lookup.get(token) {
                let def = &self.defs[idx];
                if !present.insert(idx) {
                    failures.push(ParseFailure::DuplicateOption(def.long_name.clone()));
                }
                if def.is_flag {
                    i += 1;
                } else {
                    match tokens.get(i + 1) {
                        Some(value) if !value.starts_with('-') => {
                            if value_is_invalid(def, value) {
                                failures.push(ParseFailure::InvalidValue(def.long_name.clone()));
                            }
                            i += 2;
                        }
                        _ => {
                            failures.push(ParseFailure::MissingValue(def.long_name.clone()));
                            i += 1;
                        }
                    }
                }
            } else if token.starts_with('-') {
                failures.push(ParseFailure::UnknownOption(token.clone()));
                i += 1;
            } else {
                failures.push(ParseFailure::UnexpectedArgument(token.clone()));
                i += 1;
            }
        }

        // 3. Missing-required failures, by registration order.
        for (idx, def) in self.defs.iter().enumerate() {
            if def.is_required && !present.contains(&idx) {
                failures.push(ParseFailure::MissingRequired(def.long_name.clone()));
            }
        }

        ValidationReport { failures }
    }
}

/// Returns `true` when `def` has a validation rule that rejects `value`.
fn value_is_invalid(def: &OptionDef, value: &str) -> bool {
    def.validation
        .as_ref()
        .is_some_and(|validate| !validate(value))
}

/// Collect the process's command-line tokens, skipping the program name.
///
/// This is a convenience for production use; tests and deterministic callers
/// should pass their own token slices to [`Registry::read`] and
/// [`Registry::validate`] instead.
///
/// # Examples
///
/// ```
/// // Typically called at the top of `main`:
/// let tokens = webe_args::env_tokens();
/// let _ = tokens; // pass `&tokens` to a `Registry`
/// ```
pub fn env_tokens() -> Vec<String> {
    std::env::args().skip(1).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    // Scenario 1 / C2 — read a value option.
    #[test]
    fn c2_read_value_option() {
        let mut registry = Registry::new();
        registry.add(OptionDef::value("port"));
        let tokens = t(&["--port", "8080"]);
        assert_eq!(
            registry.read("port", &tokens).unwrap(),
            OptionResult::Value("8080".to_string())
        );
    }

    // Scenario 2 / C4 — optional omitted is a successful absent result.
    #[test]
    fn c4_optional_absent() {
        let mut registry = Registry::new();
        registry.add(OptionDef::value("port"));
        assert_eq!(registry.read("port", &[]).unwrap(), OptionResult::Absent);
    }

    // Scenario 3 / C3 — flag present via long form.
    #[test]
    fn c3_flag_present_long() {
        let mut registry = Registry::new();
        registry.add(OptionDef::flag("verbose"));
        let tokens = t(&["--verbose"]);
        assert_eq!(
            registry.read("verbose", &tokens).unwrap(),
            OptionResult::Flag
        );
    }

    // Scenario 4 / C3 + C9 — flag present via short alias.
    #[test]
    fn c3_c9_flag_present_short() {
        let mut registry = Registry::new();
        registry.add(OptionDef::flag("verbose").short("v"));
        let tokens = t(&["-v"]);
        assert_eq!(
            registry.read("verbose", &tokens).unwrap(),
            OptionResult::Flag
        );
    }

    // C3 — a flag does not consume a following unrelated token.
    #[test]
    fn c3_flag_does_not_consume_following_token() {
        let mut registry = Registry::new();
        registry.add(OptionDef::flag("verbose"));
        let tokens = t(&["--verbose", "leftover"]);
        // The flag reads as present; the trailing token is not its value.
        assert_eq!(
            registry.read("verbose", &tokens).unwrap(),
            OptionResult::Flag
        );
        // And full validation flags the leftover as unexpected.
        let report = registry.validate(&tokens);
        assert_eq!(
            report.failures(),
            &[ParseFailure::UnexpectedArgument("leftover".to_string())]
        );
    }

    // Scenario 5 / C2 + NF3 — value preserved verbatim, including spaces.
    #[test]
    fn c2_nf3_value_verbatim() {
        let mut registry = Registry::new();
        registry.add(OptionDef::value("name"));
        let tokens = t(&["--name", "webe is great"]);
        assert_eq!(
            registry.read("name", &tokens).unwrap(),
            OptionResult::Value("webe is great".to_string())
        );
    }

    // Scenario 6 / C5 — missing required.
    #[test]
    fn c5_missing_required() {
        let mut registry = Registry::new();
        registry.add(OptionDef::value("port").required());
        let report = registry.validate(&[]);
        assert_eq!(
            report.failures(),
            &[ParseFailure::MissingRequired("port".to_string())]
        );
    }

    // Scenario 7 / C6 — missing value at end of input.
    #[test]
    fn c6_missing_value_end() {
        let mut registry = Registry::new();
        registry.add(OptionDef::value("port"));
        let tokens = t(&["--port"]);
        let report = registry.validate(&tokens);
        assert_eq!(
            report.failures(),
            &[ParseFailure::MissingValue("port".to_string())]
        );
    }

    // Scenario 8 / C6 — missing value when next token is dash-prefixed.
    #[test]
    fn c6_missing_value_dash_next() {
        let mut registry = Registry::new();
        registry
            .add(OptionDef::value("port"))
            .add(OptionDef::flag("verbose"));
        let tokens = t(&["--port", "--verbose"]);
        let report = registry.validate(&tokens);
        // `--verbose` is treated as an option, so `--port` is missing its value.
        assert_eq!(
            report.failures(),
            &[ParseFailure::MissingValue("port".to_string())]
        );
    }

    // Scenario 9 / C7 — invalid value, and the passing value succeeds.
    #[test]
    fn c7_invalid_value() {
        let mut registry = Registry::new();
        registry.add(OptionDef::value("port").validate(|v| v.parse::<u16>().is_ok()));

        let bad = t(&["--port", "notanumber"]);
        assert_eq!(
            registry.read("port", &bad),
            Err(ParseFailure::InvalidValue("port".to_string()))
        );

        let good = t(&["--port", "8080"]);
        assert_eq!(
            registry.read("port", &good).unwrap(),
            OptionResult::Value("8080".to_string())
        );
    }

    // Scenario 10 / C7 — valid value passes full validation.
    #[test]
    fn c7_valid_value_passes() {
        let mut registry = Registry::new();
        registry.add(OptionDef::value("port").validate(|v| v.parse::<u16>().is_ok()));
        assert!(registry.validate(&t(&["--port", "8080"])).is_success());
    }

    // Scenario 11 / C10 — unknown option named by offending token.
    #[test]
    fn c10_unknown_option() {
        let mut registry = Registry::new();
        registry.add(OptionDef::value("port"));
        let report = registry.validate(&t(&["--bogus", "x"]));
        // `--bogus` is unknown; `x` is then an unexpected positional.
        assert_eq!(
            report.failures(),
            &[
                ParseFailure::UnknownOption("--bogus".to_string()),
                ParseFailure::UnexpectedArgument("x".to_string()),
            ]
        );
    }

    // Scenario 12 / C11 — duplicate via repeated long name.
    #[test]
    fn c11_duplicate_repeated_long() {
        let mut registry = Registry::new();
        registry.add(OptionDef::value("port"));
        let report = registry.validate(&t(&["--port", "1", "--port", "2"]));
        assert_eq!(
            report.failures(),
            &[ParseFailure::DuplicateOption("port".to_string())]
        );
    }

    // Scenario 13 / C11 — duplicate via long + short for the same option.
    #[test]
    fn c11_duplicate_long_and_short() {
        let mut registry = Registry::new();
        registry.add(OptionDef::flag("verbose").short("v"));
        let report = registry.validate(&t(&["--verbose", "-v"]));
        assert_eq!(
            report.failures(),
            &[ParseFailure::DuplicateOption("verbose".to_string())]
        );
    }

    // Scenario 14 / C12 — unexpected positional token.
    #[test]
    fn c12_unexpected_positional() {
        let registry = Registry::new();
        let report = registry.validate(&t(&["extra"]));
        assert_eq!(
            report.failures(),
            &[ParseFailure::UnexpectedArgument("extra".to_string())]
        );
    }

    // Scenario 15 / C13 — conflicting definitions detected at validation time.
    #[test]
    fn c13_conflicting_definition() {
        let mut registry = Registry::new();
        registry
            .add(OptionDef::value("port"))
            .add(OptionDef::value("port"));
        let report = registry.validate(&[]);
        assert_eq!(
            report.failures(),
            &[ParseFailure::ConflictingDefinition("port".to_string())]
        );
    }

    // C13 — conflicting short aliases.
    #[test]
    fn c13_conflicting_short_alias() {
        let mut registry = Registry::new();
        registry
            .add(OptionDef::flag("verbose").short("v"))
            .add(OptionDef::flag("version").short("v"));
        let report = registry.validate(&[]);
        assert_eq!(
            report.failures(),
            &[ParseFailure::ConflictingDefinition("v".to_string())]
        );
    }

    // Scenario 16 / C8 — undeclared lookup is a distinct programming error.
    #[test]
    fn c8_undeclared_lookup() {
        let registry = Registry::new();
        assert_eq!(
            registry.read("undeclared", &[]),
            Err(ParseFailure::UndeclaredLookup("undeclared".to_string()))
        );
    }

    // C9 — long and short forms yield identical results.
    #[test]
    fn c9_long_short_equivalence() {
        let mut registry = Registry::new();
        registry.add(OptionDef::value("port").short("p"));
        let via_long = registry.read("port", &t(&["--port", "8080"])).unwrap();
        let via_short = registry.read("port", &t(&["-p", "8080"])).unwrap();
        assert_eq!(via_long, via_short);
        assert_eq!(via_long, OptionResult::Value("8080".to_string()));
    }

    // C14 — deterministic order: conflicts, then input by position, then missing.
    #[test]
    fn c14_deterministic_full_order() {
        let mut registry = Registry::new();
        registry
            .add(OptionDef::value("port").required())
            .add(OptionDef::value("port")) // conflicting definition
            .add(OptionDef::value("host").required());

        let tokens = t(&["--bogus", "stray"]);
        let report = registry.validate(&tokens);
        assert_eq!(
            report.failures(),
            &[
                // (1) conflicting definitions first
                ParseFailure::ConflictingDefinition("port".to_string()),
                // (2) input failures by left-to-right position
                ParseFailure::UnknownOption("--bogus".to_string()),
                ParseFailure::UnexpectedArgument("stray".to_string()),
                // (3) missing-required by registration order
                ParseFailure::MissingRequired("port".to_string()),
                ParseFailure::MissingRequired("host".to_string()),
            ]
        );
    }

    // Display messages name the affected option/token.
    #[test]
    fn display_messages_are_actionable() {
        assert_eq!(
            ParseFailure::MissingRequired("port".to_string()).to_string(),
            "missing required option '--port'"
        );
        assert_eq!(
            ParseFailure::UnknownOption("--bogus".to_string()).to_string(),
            "unknown option '--bogus'"
        );
    }
}
