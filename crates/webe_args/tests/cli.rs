//! Default-harness integration tests for `webe_args`.
//!
//! These tests exercise the public API over caller-supplied token slices, so they
//! run under the standard `cargo test` harness without any manual command-line
//! input. They cover the User Story 1 read paths, the all-failures
//! deterministic-order report (quickstart scenario 17), and a traceability pass
//! confirming every quickstart scenario (1–17) maps to a passing assertion.

use webe_args::{OptionDef, OptionResult, ParseFailure, Registry};

/// Helper: build a `Vec<String>` token slice from string literals.
fn tokens(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

/// Registry used across the read-path tests: a required, validated value option
/// with a short alias, an optional value option, and a flag with a short alias.
fn sample_registry() -> Registry {
    let mut registry = Registry::new();
    registry
        .add(
            OptionDef::value("port")
                .short("p")
                .required()
                .validate(|v| v.parse::<u16>().is_ok()),
        )
        .add(OptionDef::value("name"))
        .add(OptionDef::flag("verbose").short("v"));
    registry
}

// --- User Story 1: read paths over supplied tokens ---

#[test]
fn reads_value_option() {
    let registry = sample_registry();
    let tokens = tokens(&["--port", "8080"]);
    assert_eq!(
        registry.read("port", &tokens).unwrap(),
        OptionResult::Value("8080".to_string())
    );
}

#[test]
fn reads_flag_option() {
    let registry = sample_registry();
    let tokens = tokens(&["--verbose"]);
    assert_eq!(
        registry.read("verbose", &tokens).unwrap(),
        OptionResult::Flag
    );
}

#[test]
fn reads_flag_via_short_alias() {
    let registry = sample_registry();
    let tokens = tokens(&["-v"]);
    assert_eq!(
        registry.read("verbose", &tokens).unwrap(),
        OptionResult::Flag
    );
}

#[test]
fn omitted_optional_is_absent() {
    let registry = sample_registry();
    assert_eq!(
        registry.read("name", &tokens(&[])).unwrap(),
        OptionResult::Absent
    );
}

#[test]
fn value_is_preserved_verbatim() {
    let registry = sample_registry();
    let tokens = tokens(&["--name", "webe is great"]);
    assert_eq!(
        registry.read("name", &tokens).unwrap(),
        OptionResult::Value("webe is great".to_string())
    );
}

// --- Scenario 17: all failures reported in deterministic order ---

#[test]
fn reports_all_failures_in_deterministic_order() {
    let mut registry = Registry::new();
    registry.add(OptionDef::value("port").required());

    let tokens = tokens(&["--bogus", "extra", "--port"]);
    let report = registry.validate(&tokens);

    assert_eq!(
        report.failures(),
        &[
            ParseFailure::UnknownOption("--bogus".to_string()),
            ParseFailure::UnexpectedArgument("extra".to_string()),
            ParseFailure::MissingValue("port".to_string()),
        ]
    );
}

// --- Traceability: every quickstart scenario (1–17) maps to a passing check ---

#[test]
fn quickstart_scenarios_are_covered() {
    // Scenario 1: required value present -> read value.
    {
        let mut r = Registry::new();
        r.add(OptionDef::value("port").required());
        assert_eq!(
            r.read("port", &tokens(&["--port", "8080"])).unwrap(),
            OptionResult::Value("8080".to_string())
        );
    }

    // Scenario 2: optional omitted -> absent.
    {
        let mut r = Registry::new();
        r.add(OptionDef::value("opt"));
        assert_eq!(r.read("opt", &tokens(&[])).unwrap(), OptionResult::Absent);
    }

    // Scenario 3: flag present (long form).
    {
        let mut r = Registry::new();
        r.add(OptionDef::flag("verbose"));
        assert_eq!(
            r.read("verbose", &tokens(&["--verbose"])).unwrap(),
            OptionResult::Flag
        );
    }

    // Scenario 4: flag via short alias (same as long).
    {
        let mut r = Registry::new();
        r.add(OptionDef::flag("verbose").short("v"));
        assert_eq!(
            r.read("verbose", &tokens(&["-v"])).unwrap(),
            OptionResult::Flag
        );
    }

    // Scenario 5: value preserved verbatim.
    {
        let mut r = Registry::new();
        r.add(OptionDef::value("name"));
        assert_eq!(
            r.read("name", &tokens(&["--name", "webe is great"]))
                .unwrap(),
            OptionResult::Value("webe is great".to_string())
        );
    }

    // Scenario 6: missing required.
    {
        let mut r = Registry::new();
        r.add(OptionDef::value("port").required());
        assert_eq!(
            r.validate(&tokens(&[])).failures(),
            &[ParseFailure::MissingRequired("port".to_string())]
        );
    }

    // Scenario 7: missing value (end of input).
    {
        let mut r = Registry::new();
        r.add(OptionDef::value("port"));
        assert_eq!(
            r.validate(&tokens(&["--port"])).failures(),
            &[ParseFailure::MissingValue("port".to_string())]
        );
    }

    // Scenario 8: missing value (dash-prefixed next token).
    {
        let mut r = Registry::new();
        r.add(OptionDef::value("port"))
            .add(OptionDef::flag("verbose"));
        assert_eq!(
            r.validate(&tokens(&["--port", "--verbose"])).failures(),
            &[ParseFailure::MissingValue("port".to_string())]
        );
    }

    // Scenario 9: invalid value.
    {
        let mut r = Registry::new();
        r.add(OptionDef::value("port").validate(|v| v.parse::<u16>().is_ok()));
        assert_eq!(
            r.read("port", &tokens(&["--port", "notanumber"])),
            Err(ParseFailure::InvalidValue("port".to_string()))
        );
    }

    // Scenario 10: valid value passes.
    {
        let mut r = Registry::new();
        r.add(OptionDef::value("port").validate(|v| v.parse::<u16>().is_ok()));
        assert!(r.validate(&tokens(&["--port", "8080"])).is_success());
    }

    // Scenario 11: unknown option.
    {
        let r = Registry::new();
        assert_eq!(
            r.validate(&tokens(&["--bogus", "x"])).failures(),
            &[
                ParseFailure::UnknownOption("--bogus".to_string()),
                ParseFailure::UnexpectedArgument("x".to_string()),
            ]
        );
    }

    // Scenario 12: duplicate (repeated long).
    {
        let mut r = Registry::new();
        r.add(OptionDef::value("port"));
        assert_eq!(
            r.validate(&tokens(&["--port", "1", "--port", "2"]))
                .failures(),
            &[ParseFailure::DuplicateOption("port".to_string())]
        );
    }

    // Scenario 13: duplicate (long + short).
    {
        let mut r = Registry::new();
        r.add(OptionDef::flag("verbose").short("v"));
        assert_eq!(
            r.validate(&tokens(&["--verbose", "-v"])).failures(),
            &[ParseFailure::DuplicateOption("verbose".to_string())]
        );
    }

    // Scenario 14: unexpected positional.
    {
        let r = Registry::new();
        assert_eq!(
            r.validate(&tokens(&["extra"])).failures(),
            &[ParseFailure::UnexpectedArgument("extra".to_string())]
        );
    }

    // Scenario 15: conflicting definitions.
    {
        let mut r = Registry::new();
        r.add(OptionDef::value("port"))
            .add(OptionDef::value("port"));
        assert_eq!(
            r.validate(&tokens(&[])).failures(),
            &[ParseFailure::ConflictingDefinition("port".to_string())]
        );
    }

    // Scenario 16: undeclared lookup.
    {
        let r = Registry::new();
        assert_eq!(
            r.read("undeclared", &tokens(&[])),
            Err(ParseFailure::UndeclaredLookup("undeclared".to_string()))
        );
    }

    // Scenario 17: all failures, deterministic order.
    {
        let mut r = Registry::new();
        r.add(OptionDef::value("port").required());
        assert_eq!(
            r.validate(&tokens(&["--bogus", "extra", "--port"]))
                .failures(),
            &[
                ParseFailure::UnknownOption("--bogus".to_string()),
                ParseFailure::UnexpectedArgument("extra".to_string()),
                ParseFailure::MissingValue("port".to_string()),
            ]
        );
    }
}
