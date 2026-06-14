# Feature Specification: Revamp Args Crate

**Feature Branch**: `001-revamp-args-crate`

**Created**: 2026-06-13

**Status**: Draft

**Input**: User description: "Create a spec for revamping the 'args' crate. The crate's purpose is to be able to handle simple command line options passed into rust programs. It needs to be well tested and well documented."

## Clarifications

### Session 2026-06-13

- Q: How should the revamped args crate handle command-line options that were not declared by the developer during full startup validation? → A: Reject unknown options with a distinct unknown-option failure that names the offending token.
- Q: When the same option is supplied more than once, including repeated long names or both long and short forms for the same option, how should the crate behave? → A: Reject duplicate occurrences with a distinct duplicate-option failure naming the option or alias.
- Q: How should the parser treat a token that starts with `-` when it appears immediately after an option that requires a value? → A: Treat the dash-prefixed token as another option token, making the preceding option fail with missing-value.
- Q: How should full startup validation handle bare non-option tokens that are not consumed as values for declared options? → A: Reject unexpected positional tokens with a distinct unexpected-argument failure naming the token.
- Q: When full startup validation finds multiple problems in the same command line, how should failures be reported? → A: Return all discovered failures in deterministic order.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Define and Read Simple Options (Priority: P1)

A developer building a small command-line application can define expected options, read the provided command-line input, and receive the correct value or flag state without writing custom parsing logic.

**Why this priority**: This is the core value of the crate. Without reliable option definition and reading, documentation and test coverage cannot deliver useful behavior.

**Independent Test**: Can be tested by defining a required value option, an optional value option, and a boolean flag, then running the parser against representative command-line input and confirming each result matches the provided input.

**Acceptance Scenarios**:

1. **Given** a declared required option that accepts a value, **When** the command line includes the option followed by a value, **Then** the developer receives that value for the option.
2. **Given** a declared optional option, **When** the command line omits the option, **Then** the developer receives a successful no-value result rather than an error.
3. **Given** a declared flag option, **When** the command line includes the flag, **Then** the developer receives a successful present result without needing to provide a value.

---

### User Story 2 - Diagnose Missing or Invalid Input (Priority: P2)

A developer can distinguish between missing required options, missing option values, invalid values, duplicate option occurrences, unknown option tokens, unexpected positional tokens, and undeclared option lookups so application startup failures are clear and actionable.

**Why this priority**: Command-line failures often happen before an application starts. Clear outcomes prevent confusing startup behavior and reduce support effort for applications using the crate.

**Independent Test**: Can be tested by defining options with required and validation rules, passing incomplete or invalid command-line input, and confirming each failure reports the correct reason and option.

**Acceptance Scenarios**:

1. **Given** a required option, **When** the command line omits it, **Then** the developer receives a missing-required-option failure that names the option.
2. **Given** a value-taking option, **When** the command line includes the option without a following value, **Then** the developer receives a missing-value failure that names the option.
3. **Given** an option with a validation rule, **When** the command line provides a value that fails validation, **Then** the developer receives an invalid-value failure that names the option.
4. **Given** startup validation receives an option token that has no declared long name or short alias, **When** the command line is validated, **Then** the developer receives an unknown-option failure that names the offending token.
5. **Given** startup validation receives the same option more than once, including repeated long names or both long and short forms, **When** the command line is validated, **Then** the developer receives a duplicate-option failure that names the option or alias.
6. **Given** a value-taking option is followed immediately by a token beginning with `-`, **When** the command line is validated, **Then** the dash-prefixed token is treated as an option token and the value-taking option receives a missing-value failure.
7. **Given** startup validation receives a bare non-option token that is not consumed as a declared option value, **When** the command line is validated, **Then** the developer receives an unexpected-argument failure that names the token.
8. **Given** startup validation finds multiple failures in the same command line, **When** validation completes, **Then** the developer receives all discovered failures in deterministic order.

---

### User Story 3 - Use Long and Short Option Forms (Priority: P3)

A developer can define a readable long option name and an optional short alias, and users of the application can provide either form with the same result.

**Why this priority**: Short aliases are a common convenience, while long names keep commands self-describing. Supporting both improves ergonomics without expanding beyond simple command-line options.

**Independent Test**: Can be tested by defining an option with both a long name and short alias, then confirming either input form produces the same parsed result.

**Acceptance Scenarios**:

1. **Given** an option with a long name and short alias, **When** the command line uses the long form, **Then** the developer receives the expected result.
2. **Given** the same option, **When** the command line uses the short alias, **Then** the developer receives the same result as the long form.

---

### User Story 4 - Learn and Verify Behavior (Priority: P4)

A developer evaluating or maintaining the crate can understand the supported behaviors from documentation and trust that those behaviors are protected by repeatable automated tests.

**Why this priority**: The revamp is only useful if developers can adopt it confidently and maintainers can change it without guessing what behavior is guaranteed.

**Independent Test**: Can be tested by following the documentation examples from a clean project and by running the documented verification suite without supplying manual command-line text.

**Acceptance Scenarios**:

1. **Given** a developer who has not used the crate before, **When** they read the crate documentation, **Then** they can identify how to define value options, flags, optional options, required options, aliases, validation, and error handling.
2. **Given** a maintainer changing parser behavior, **When** they run the documented tests, **Then** the tests verify successful parsing and each documented failure mode without manual setup.

### Edge Cases

- Required value option is omitted entirely.
- Value-taking option is present as the final command-line token with no value after it.
- Optional option is omitted and must not block startup validation.
- Flag option is followed by unrelated command-line text that should not be treated as the flag's value.
- The same option appears more than once in one command line and must be rejected as a duplicate occurrence.
- Short alias and long name both appear for the same option in one command line and must be rejected as a duplicate occurrence.
- Unknown command-line options appear alongside declared options and must be rejected during full startup validation.
- A token beginning with `-` appears after a value-taking option and must be treated as an option token, not as that option's value.
- A bare non-option token appears without being consumed as a declared option value and must be rejected during full startup validation.
- Multiple validation failures appear in the same command line and must all be reported in deterministic order.
- Validation rejects an empty value, malformed value, or value outside allowed business rules.
- Multiple declared options share a conflicting short alias or long name.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The crate MUST allow developers to define command-line options with a long name, optional short alias, human-readable description, required-or-optional status, flag-or-value behavior, and optional value validation rule. Acceptance: a review can identify tests or examples covering each supported option attribute.
- **FR-002**: The crate MUST allow developers to read a declared value option from provided command-line input and receive the associated value when the option is present. Acceptance: a supplied value is returned exactly as provided for a declared option.
- **FR-003**: The crate MUST allow developers to read a declared flag option from provided command-line input and receive a successful present result without requiring or returning a value. Acceptance: a flag succeeds when present and does not consume following unrelated input as its value.
- **FR-004**: The crate MUST treat an omitted optional option as a successful no-value result. Acceptance: startup validation passes when optional options are absent and all required options are valid.
- **FR-005**: The crate MUST report an omitted required option as a distinct failure that identifies the option. Acceptance: an omitted required option produces the documented missing-required failure.
- **FR-006**: The crate MUST report a value-taking option with no provided value as a distinct failure that identifies the option, including when the next token begins with `-` and is therefore treated as an option token rather than a value. Acceptance: an option appearing at the end of input or immediately before a dash-prefixed token without its value produces the documented missing-value failure.
- **FR-007**: The crate MUST report a failed validation rule as a distinct failure that identifies the option. Acceptance: the same option can be tested with one accepted value and one rejected value.
- **FR-008**: The crate MUST report attempts to read an undeclared option as a distinct developer error. Acceptance: looking up an option that was not defined produces the documented undeclared-option failure.
- **FR-009**: The crate MUST support equivalent long-name and short-alias input for the same declared option. Acceptance: long and short forms produce the same parsed result for the same option.
- **FR-010**: The crate MUST provide a way to validate the full command line at application startup, including all declared options and all user-supplied option tokens, and reject any unknown option token that has no declared long name or short alias. Acceptance: a single validation pass can confirm a valid command line or report all discovered documented failures in deterministic order, including unknown-option failures that name offending tokens.
- **FR-011**: The crate MUST reject duplicate occurrences of the same option in one command line, including repeated long names, repeated short aliases, or a long name and short alias for the same option. Acceptance: duplicate input produces a distinct duplicate-option failure that names the option or alias.
- **FR-012**: The crate MUST reject bare non-option tokens that are not consumed as declared option values during full startup validation. Acceptance: unexpected positional input produces a distinct unexpected-argument failure that names the token.
- **FR-013**: The crate MUST prevent or report conflicting option definitions, including duplicate long names and duplicate short aliases. Acceptance: conflicts are identified before user input is accepted as valid.
- **FR-014**: Documentation MUST explain every supported option shape, success result, failure result, and common usage pattern, including examples for value options, flags, optional options, required options, aliases, validation, duplicate-option failures, dash-prefixed token behavior, unexpected positional-token behavior, and all-failures startup validation. Acceptance: every public behavior listed in this specification maps to documentation text and at least one example or scenario.
- **FR-015**: Automated tests MUST cover every documented behavior, including successful parsing, missing required options, missing values, dash-prefixed tokens after value options, invalid values, optional omissions, aliases, flags, duplicate option input, unexpected positional input, all-failures validation reporting, definition conflicts, and undeclared option lookups. Acceptance: each documented behavior can be traced to at least one automated test.
- **FR-016**: Tests MUST be repeatable through the normal project verification flow without requiring a maintainer to manually pass custom command-line text. Acceptance: a maintainer can run the documented verification command from a clean checkout and observe the args crate tests without custom invocation arguments.

### Key Entities *(include if feature involves data)*

- **Option Definition**: Represents one accepted command-line option, including its long name, optional short alias, description, required status, whether it is a flag or value option, and any validation rule.
- **Command-Line Input**: Represents the ordered option names, aliases, flags, and values supplied when an application starts.
- **Option Result**: Represents the outcome of reading a declared option, including a value, flag presence, successful absence for optional options, or a failure.
- **Parse Failure**: Represents a distinct problem that prevents successful option use, such as a missing required option, missing value, invalid value, duplicate option occurrence, unknown command-line option, unexpected positional argument, undeclared lookup, or conflicting definition.
- **Validation Report**: Represents the full startup validation outcome, including success or all discovered parse failures in deterministic order.
- **Usage Documentation**: Represents the published guidance and examples developers use to understand and adopt the crate.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer new to the crate can define and read one required value option, one optional value option, and one flag in under 10 minutes using only the published documentation.
- **SC-002**: 100% of documented option behaviors have automated tests covering the expected success path and relevant failure paths before the revamp is considered complete.
- **SC-003**: During acceptance review, every documented failure mode identifies the affected option or token and reason for failure without requiring maintainers to inspect parser internals, and multi-failure command lines report all discovered failures in deterministic order.
- **SC-004**: Documentation examples for the primary usage patterns can be followed successfully from a clean project with zero stale, missing, or contradictory steps.
- **SC-005**: A command line containing at least 20 declared options and 100 supplied tokens can be validated during application startup without a user-noticeable delay.
- **SC-006**: The revamp leaves the crate with no undocumented public behavior in the supported simple-option scope.

## Assumptions

- Primary users are developers adding simple command-line configuration to small applications and examples in the Webe toolkit ecosystem.
- The revamp scope is limited to simple options: long names, optional short aliases, flags, single values, required-or-optional status, validation, clear failures, tests, and documentation.
- Positional arguments, subcommands, shell completion, environment-variable merging, configuration files, interactive prompts, and rich typed deserialization are outside the first revamp scope unless added by a later spec.
- Improving correctness, documentation, and testability may justify documented breaking changes, but any breaking change must be visible to developers before release.
- The crate remains discoverable through the existing toolkit documentation and facade expectations.
