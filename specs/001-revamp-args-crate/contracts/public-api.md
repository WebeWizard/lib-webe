# Public API Contract: webe_args

The crate's external interface is its public Rust API (a library), surfaced both
directly as `webe_args` and through the facade re-export `webe::args`. This
contract defines the observable behavior the implementation MUST satisfy. Exact
type names, signatures, and ownership are finalized in implementation; the
behavioral guarantees below are binding and map to the spec's functional
requirements.

## Surface overview

- A way to construct an option **registry** and **add option definitions** to it.
- A way to **validate option definitions** (detecting conflicts) — FR-013.
- A way to **validate a full command line** of caller-supplied tokens, returning a
  **validation report** (success or all failures in deterministic order) — FR-010.
- A way to **read a single declared option's result** from supplied tokens — FR-002,
  FR-003, FR-004, FR-008.
- A typed **failure** describing each documented problem — FR-005..FR-012.

> Core entry points accept caller-supplied tokens (e.g. `&[String]` / iterator) so
> behavior is deterministic and testable without the process environment. A
> convenience path MAY read process arguments.

## Behavioral contract

### C1 — Define options (FR-001)
Given a registry, when the developer adds an option with long name, optional short
alias, description, required/optional, flag/value, and optional validation, then
the definition is accepted and recognized for later reads and validation.

### C2 — Read value option (FR-002)
Given a declared value option present in the tokens followed by a value, when read,
then the result is the value returned exactly as supplied (no mutation).

### C3 — Read flag option (FR-003)
Given a declared flag option present in the tokens, when read, then the result is a
present-flag outcome with no value; the flag MUST NOT consume an unrelated
following token as its value.

### C4 — Optional omitted (FR-004)
Given a declared optional option absent from the tokens, when read, then the result
is a successful no-value (absent) outcome, and full validation passes if all
required options are valid.

### C5 — Missing required (FR-005)
Given a declared required option absent from the tokens, when validated, then a
`MissingRequired` failure naming the option is produced.

### C6 — Missing value, including dash-prefixed next token (FR-006)
Given a value option that is the final token, or is immediately followed by a token
beginning with `-`, when validated, then the dash-prefixed token is treated as an
option token and the value option produces a `MissingValue` failure naming the
option.

### C7 — Invalid value (FR-007)
Given a value option with a validation rule and a value that fails the rule, when
validated, then an `InvalidValue` failure naming the option is produced. The same
option with a passing value succeeds.

### C8 — Undeclared lookup (FR-008)
Given a read for an option name that was never defined, when read, then a distinct
undeclared/programming-error result is produced (not a normal value/absent result).

### C9 — Long/short equivalence (FR-009)
Given an option with a long name and short alias, when either form is supplied,
then the parsed result is identical.

### C10 — Full validation rejects unknown options (FR-010)
Given tokens containing an option token with no declared long name or alias, when
the full command line is validated, then the report contains an `UnknownOption`
failure naming the offending token.

### C11 — Reject duplicates (FR-011)
Given the same option supplied more than once (repeated long names, repeated
aliases, or long+short for the same option), when validated, then a
`DuplicateOption` failure naming the option or alias is produced.

### C12 — Reject unexpected positionals (FR-012)
Given a bare non-option token not consumed as a declared option value, when
validated, then an `UnexpectedArgument` failure naming the token is produced.

### C13 — Reject conflicting definitions (FR-013)
Given two definitions sharing a long name or short alias, when definitions are
validated, then a `ConflictingDefinition` failure naming the option or alias is
produced before user input is accepted as valid.

### C14 — All failures, deterministic order (FR-010 clarification, SC-003)
Given a command line with multiple problems, when validated, then the report
contains all discovered failures in deterministic order: conflicting definitions
first, then input failures by left-to-right token position, then missing-required
failures by option registration order.

### C15 — Facade reachability (FR-012 spec / Constitution III)
The crate's functionality MUST remain reachable via `webe::args` when the `args`
feature is enabled.

## Non-functional contract

- **NF1 (Performance, SC-005)**: Validating ~20 declared options and ~100 tokens
  completes with no user-noticeable startup delay; complexity is linear in option
  and token counts.
- **NF2 (Quality, Constitution I)**: No `panic!`/`unwrap`/`expect` in library
  paths; no `unsafe`; all public items documented; lint-clean.
- **NF3 (Stability of values)**: Returned values preserve original text, including
  spaces and punctuation (C2).

## Verification mapping

Every contract item above MUST be covered by at least one automated test
(FR-015) runnable via `cargo test` without manual command-line input (FR-016).
See `quickstart.md` for the runnable validation scenarios.
