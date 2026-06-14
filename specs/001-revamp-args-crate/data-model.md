# Phase 1 Data Model: Revamp Args Crate

Derived from the spec's Key Entities and Functional Requirements. Field names are
conceptual; concrete Rust types are finalized during implementation. The crate
stays dependency-free (`std` only).

## Entity: Option Definition

One accepted command-line option.

| Field | Type (conceptual) | Notes |
|-------|-------------------|-------|
| `long_name` | string | Required. Declared identity, used as `--long_name`. |
| `short_alias` | optional string | Optional. Used as `-a`. |
| `description` | optional string | Human-readable help text. |
| `is_required` | bool | Whether absence is a failure. |
| `is_flag` | bool | Flag (no value) vs value-taking. |
| `validation` | optional rule | Predicate over the value string. |

**Validation rules (definition-time, FR-013):**
- `long_name` MUST be non-empty.
- `short_alias`, when present, MUST be non-empty.
- A flag (`is_flag = true`) MUST NOT also be value-taking; a flag has no value and
  no value validation is applied.
- Across all definitions, `long_name` values MUST be unique and `short_alias`
  values MUST be unique (no duplicate long names or short aliases).

**Relationships**: Owned by the parser/registry; many definitions per registry.

## Entity: Command-Line Input

The ordered tokens supplied to validation (caller-provided, not read from the live
process environment in core logic).

| Field | Type (conceptual) | Notes |
|-------|-------------------|-------|
| `tokens` | ordered list of strings | Option names, aliases, flags, and values. |

**Token interpretation rules:**
- A token equal to a declared `--long_name` or `-short_alias` is an option token.
- A token beginning with `-` is treated as an option token, never as a value for a
  preceding value option (FR-006 clarification).
- A non-option token immediately following a value option is that option's value.
- Any other bare non-option token is an unexpected positional token (FR-012).

## Entity: Option Result

Outcome of reading one declared option from the input.

| Variant | Meaning |
|---------|---------|
| `Value(string)` | A value option was present with a value. |
| `Flag` (present) | A flag option was present. |
| `Absent` | An optional option was not supplied (success, no value). |
| `Failure(ParseFailure)` | Reading failed for a documented reason. |

## Entity: Parse Failure

A distinct, developer-actionable problem. Each failure identifies the affected
option or token.

| Failure kind | Trigger | Identifies |
|--------------|---------|------------|
| `MissingRequired` | Required option absent (FR-005) | option long name |
| `MissingValue` | Value option has no value, incl. next token is `-`-prefixed or end of input (FR-006) | option long name |
| `InvalidValue` | Validation rule rejects the value (FR-007) | option long name |
| `UnknownOption` | Option token with no declared long name/alias (FR-010) | offending token |
| `DuplicateOption` | Same option supplied more than once (FR-011) | option or alias |
| `UnexpectedArgument` | Bare positional token not consumed as a value (FR-012) | offending token |
| `ConflictingDefinition` | Duplicate long name / short alias at definition time (FR-013) | option or alias |
| `UndeclaredLookup` | Developer reads an option that was never defined (FR-008) | requested name |

`UndeclaredLookup` is a developer/programming error (looking up a name that was
never registered); the others arise from definitions or user-supplied input.

## Entity: Validation Report

Result of full startup validation over the whole command line (FR-010).

| Field | Type (conceptual) | Notes |
|-------|-------------------|-------|
| outcome | success or failure list | Failure variant carries all discovered failures. |
| `failures` | ordered list of ParseFailure | Deterministic order (see below). |

**Deterministic ordering (FR-010, SC-003):**
1. Definition-level `ConflictingDefinition` failures first (detected before input parsing).
2. Then input-derived failures ordered by left-to-right token position.
3. `MissingRequired` failures (which have no token position) ordered by option
   registration order, after positional failures.

## Entity: Usage Documentation

Crate-level rustdoc + README content and runnable examples that map every public
behavior to explanatory text and at least one example (FR-014, SC-004, SC-006).

## State / Lifecycle

```text
define options ──▶ validate definitions ──▶ validate command line ──▶ read option results
     │                    │                          │                         │
     ▼                    ▼                          ▼                         ▼
 register names   ConflictingDefinition?   collect all input failures   Value / Flag / Absent / Failure
                  (fail fast at def time)  (deterministic order)
```
