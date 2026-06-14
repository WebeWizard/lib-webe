# webe_args

A small, dependency-free parser for simple command-line options.

`webe_args` lets you declare named options and then either read a single option
or validate a whole command line in one pass. Core entry points accept a
caller-supplied token slice (`&[String]`) instead of reading the process
environment, so behavior is deterministic and easy to test. The crate is
`std`-only, contains no `unsafe`, and reports problems as typed
[`ParseFailure`](#failure-modes) values rather than panicking.

## Scope

Supported: long names, optional short aliases, descriptions, required/optional
status, flags (presence-only) vs. value options, optional value validation, full
command-line validation, and typed failures.

Out of scope (for now): positional arguments, subcommands, shell completion,
environment-variable merging, config files, interactive prompts, and typed
deserialization.

## Quick start

```rust
use webe_args::{OptionDef, OptionResult, Registry};

let mut registry = Registry::new();
registry
    .add(
        OptionDef::value("port")
            .short("p")
            .description("TCP port to bind")
            .required()
            .validate(|v| v.parse::<u16>().is_ok()),
    )
    .add(OptionDef::flag("verbose").short("v"));

let tokens = vec!["--port".to_string(), "8080".to_string(), "-v".to_string()];

// Validate the whole command line up front.
let report = registry.validate(&tokens);
assert!(report.is_success());

// Read individual options.
assert_eq!(registry.read("port", &tokens).unwrap(), OptionResult::Value("8080".to_string()));
assert_eq!(registry.read("verbose", &tokens).unwrap(), OptionResult::Flag);
```

In production, obtain the live tokens with the convenience helper:

```rust
let tokens = webe_args::env_tokens(); // std::env::args() minus the program name
let _ = tokens; // pass `&tokens` to your `Registry`
```

## Defining options

Every option starts from one of two constructors and is refined with chainable
builder methods:

| Shape | How to declare |
|-------|----------------|
| Value option | `OptionDef::value("name")` |
| Flag (no value) | `OptionDef::flag("verbose")` |
| Short alias | `.short("n")` |
| Description | `.description("help text")` |
| Required | `.required()` (optional by default) |
| Validation | `.validate(|v| /* bool */)` |

```rust
use webe_args::OptionDef;

let required_value = OptionDef::value("config").required();
let optional_value = OptionDef::value("name");
let flag          = OptionDef::flag("verbose").short("v");
let validated     = OptionDef::value("port").validate(|v| v.parse::<u16>().is_ok());
```

## Token interpretation

- A token equal to a declared `--long_name` or `-short_alias` is an option token.
- Any token beginning with `-` is treated as an option token, **never** as the
  value of a preceding value option.
- A bare (non-`-`) token immediately following a value option is that option's
  value, returned **verbatim** (no trimming or mutation).
- Any other bare token is an unexpected positional argument.

```rust
use webe_args::{OptionDef, ParseFailure, Registry};

let mut registry = Registry::new();
registry.add(OptionDef::value("port"));

// `--port` is followed by a dash-prefixed token, so it has no value.
let report = registry.validate(&["--port".to_string(), "--other".to_string()]);
assert_eq!(report.failures()[0], ParseFailure::MissingValue("port".to_string()));
```

## Reading a single option

`Registry::read` returns `Result<OptionResult, ParseFailure>`:

| Outcome | Meaning |
|---------|---------|
| `Ok(OptionResult::Value(v))` | Value option present; `v` is verbatim. |
| `Ok(OptionResult::Flag)` | Flag present. |
| `Ok(OptionResult::Absent)` | Optional option absent (success). |
| `Err(ParseFailure::MissingRequired)` | Required option absent. |
| `Err(ParseFailure::MissingValue)` | Value option had no value. |
| `Err(ParseFailure::InvalidValue)` | Validation rejected the value. |
| `Err(ParseFailure::UndeclaredLookup)` | The name was never defined (a bug). |

```rust
use webe_args::{OptionDef, OptionResult, ParseFailure, Registry};

let mut registry = Registry::new();
registry.add(OptionDef::value("name"));

assert_eq!(
    registry.read("name", &["--name".to_string(), "webe".to_string()]).unwrap(),
    OptionResult::Value("webe".to_string())
);
assert_eq!(registry.read("name", &[]).unwrap(), OptionResult::Absent);
assert_eq!(
    registry.read("missing", &[]),
    Err(ParseFailure::UndeclaredLookup("missing".to_string()))
);
```

A flag never consumes a following token:

```rust
use webe_args::{OptionDef, OptionResult, Registry};

let mut registry = Registry::new();
registry.add(OptionDef::flag("verbose"));

let tokens = vec!["--verbose".to_string(), "leftover".to_string()];
assert_eq!(registry.read("verbose", &tokens).unwrap(), OptionResult::Flag);
```

## Validating the whole command line

`Registry::validate` walks the definitions and all tokens once and returns a
`ValidationReport`. It collects **every** failure (it does not stop at the
first) and orders them deterministically:

1. `ConflictingDefinition` failures (detected before input parsing).
2. Input-derived failures, by left-to-right token position.
3. `MissingRequired` failures, by option registration order.

```rust
use webe_args::{OptionDef, ParseFailure, Registry};

let mut registry = Registry::new();
registry.add(OptionDef::value("port").required());

let tokens = vec!["--bogus".to_string(), "extra".to_string(), "--port".to_string()];
let report = registry.validate(&tokens);

assert_eq!(
    report.failures(),
    &[
        ParseFailure::UnknownOption("--bogus".to_string()),
        ParseFailure::UnexpectedArgument("extra".to_string()),
        ParseFailure::MissingValue("port".to_string()),
    ]
);
```

## Failure modes

`ParseFailure` implements `Debug`, `Display` (actionable messages), and
`std::error::Error`. Each variant names the affected option (by long name) or the
offending token:

| Variant | Trigger | Identifies |
|---------|---------|------------|
| `MissingRequired` | Required option absent | option long name |
| `MissingValue` | Value option with no value (end of input or `-`-prefixed next token) | option long name |
| `InvalidValue` | Validation rule returned `false` | option long name |
| `UnknownOption` | Option token with no declared long name/alias | offending token |
| `DuplicateOption` | Same option supplied more than once (repeated long, repeated short, or long + short) | option long name |
| `UnexpectedArgument` | Bare positional token not consumed as a value | offending token |
| `ConflictingDefinition` | Two definitions share a long name or short alias | name or alias |
| `UndeclaredLookup` | A read requested an option that was never defined | requested name |

Examples for each failure:

```rust
use webe_args::{OptionDef, ParseFailure, Registry};

// Duplicate option.
let mut r = Registry::new();
r.add(OptionDef::value("port"));
assert_eq!(
    r.validate(&["--port".to_string(), "1".to_string(), "--port".to_string(), "2".to_string()]).failures(),
    &[ParseFailure::DuplicateOption("port".to_string())]
);

// Unexpected positional.
let r = Registry::new();
assert_eq!(
    r.validate(&["extra".to_string()]).failures(),
    &[ParseFailure::UnexpectedArgument("extra".to_string())]
);

// Conflicting definition.
let mut r = Registry::new();
r.add(OptionDef::value("port")).add(OptionDef::value("port"));
assert_eq!(
    r.validate(&[]).failures(),
    &[ParseFailure::ConflictingDefinition("port".to_string())]
);
```

## Long and short equivalence

An option's long name and short alias resolve to the same definition and yield
identical results:

```rust
use webe_args::{OptionDef, Registry};

let mut registry = Registry::new();
registry.add(OptionDef::value("port").short("p"));

let via_long = registry.read("port", &["--port".to_string(), "8080".to_string()]).unwrap();
let via_short = registry.read("port", &["-p".to_string(), "8080".to_string()]).unwrap();
assert_eq!(via_long, via_short);
```

## Testing

All behaviors are covered by unit tests (`src/lib.rs`), integration tests
(`tests/cli.rs`), and runnable doctests. Run them with the standard harness — no
manual command-line input required:

```bash
cargo test -p webe_args          # unit + integration + doctests
cargo test -p webe_args --doc     # documentation examples only
```

## Breaking changes from the previous design

This release **replaces** (does not extend) the old API. Migrate as follows:

| Old (removed) | New |
|---------------|-----|
| `Args::new()` | `Registry::new()` |
| `args.add(name, ArgOpts { .. })` | `registry.add(OptionDef::value("name")...)` / `OptionDef::flag("name")...` |
| `ArgOpts { short, description, is_required, is_flag, validation }` | `OptionDef` builder: `.short()`, `.description()`, `.required()`, `OptionDef::flag()`, `.validate()` |
| `DEFAULT_OPTS` constant | Builder defaults (optional value option) |
| `ArgError` (`NoArgOpt`, `ArgNotFound`, `RequiredNotFound`, `ValueNotFound`, `InvalidValue`) | `ParseFailure` (8 typed variants naming the option/token) |
| `args.get(name) -> Result<Option<String>, ArgError>` | `registry.read(name, &tokens) -> Result<OptionResult, ParseFailure>` |
| `args.parse_args()` — **panics** on bad input, reads `std::env::args()` | `registry.validate(&tokens) -> ValidationReport` — returns all failures, no panics |
| `format_as_long` / `format_as_short` (public helpers) | Internal; not part of the public API |

Key behavioral differences:

- **No panics.** Bad input is reported via `ParseFailure` / `ValidationReport`.
- **Caller-supplied tokens.** Core logic takes `&[String]`; use `env_tokens()`
  for the live process arguments.
- **All failures, ordered.** `validate` returns every problem in deterministic
  order instead of failing fast.
- **New rejections.** Unknown options, unexpected positionals, duplicate
  occurrences, and conflicting definitions are now distinct, named failures.
- **Verbatim values.** Returned values preserve the original text exactly.
