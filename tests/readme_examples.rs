//! Runs every `calc "expr"  # expected` example from the `### Examples`
//! section of README.md against the real evaluator, so the README can never
//! silently drift out of sync with what the crate actually does.
//!
//! Expected-value format in README.md:
//! - a plain number (`8`, `-2`, `0.5`) is compared exactly;
//! - a `~`-prefixed number (`~1.5708`) is compared with a small tolerance,
//!   for irrational results shown rounded for readability;
//! - a double-quoted string (`"ff"`) asserts the result is that exact text
//!   (used for the base-conversion functions, `hex`/`oct`/`bin`);
//! - `error: <message>` asserts evaluation fails with that exact message
//!   (matching the CLI's `error: {e}` output format).
//!
//! A trailing parenthetical comment, e.g. `(power)`, is informational only
//! and stripped before comparison.

use std::fs;
use std::path::Path;

/// One `calc "expr"  # expected` line extracted from README.md.
struct Example {
    expr: String,
    expected: String,
}

fn parse_readme_examples(readme: &str) -> Vec<Example> {
    let mut examples = Vec::new();
    for line in readme.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("calc ") else {
            continue;
        };
        let rest = rest.strip_prefix("-- ").unwrap_or(rest);

        let Some(q1) = rest.find('"') else { continue };
        let after_q1 = &rest[q1 + 1..];
        let Some(q2) = after_q1.find('"') else {
            continue;
        };
        let expr = &after_q1[..q2];
        let after_expr = &after_q1[q2 + 1..];
        let Some(hash) = after_expr.find('#') else {
            continue;
        };

        let mut expected = after_expr[hash + 1..].trim().to_string();
        if let Some(paren) = expected.find('(') {
            expected = expected[..paren].trim().to_string();
        }

        examples.push(Example {
            expr: expr.to_string(),
            expected,
        });
    }
    examples
}

#[test]
fn readme_examples_match_evaluator() {
    let readme_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
    let readme = fs::read_to_string(&readme_path).expect("README.md should be readable");

    let examples = parse_readme_examples(&readme);
    assert!(
        examples.len() >= 30,
        "expected to find at least 30 examples in README.md's Examples section, found {}; \
         did the format change or the section get removed?",
        examples.len()
    );

    let mut failures = Vec::new();
    for example in &examples {
        let result = excalc::evaluate(&example.expr);

        if let Some(message) = example.expected.strip_prefix("error: ") {
            match result {
                Err(e) if e.to_string() == message => {}
                Err(e) => failures.push(format!(
                    "{:?}: expected error {message:?}, got error {:?}",
                    example.expr,
                    e.to_string()
                )),
                Ok(v) => failures.push(format!(
                    "{:?}: expected error {message:?}, got Ok({v})",
                    example.expr
                )),
            }
            continue;
        }

        if let Some(expected_text) = example
            .expected
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
        {
            match excalc::evaluate_value(&example.expr) {
                Ok(excalc::Value::Text(text)) if text == expected_text => {}
                Ok(v) => failures.push(format!(
                    "{:?}: expected {expected_text:?}, got {v}",
                    example.expr
                )),
                Err(e) => failures.push(format!(
                    "{:?}: expected {expected_text:?}, got error {:?}",
                    example.expr,
                    e.to_string()
                )),
            }
            continue;
        }

        let (approx, expected_str) = match example.expected.strip_prefix('~') {
            Some(rest) => (true, rest),
            None => (false, example.expected.as_str()),
        };
        let expected: f64 = expected_str.parse().unwrap_or_else(|_| {
            panic!(
                "{:?}: expected value {:?} isn't a number",
                example.expr, example.expected
            )
        });

        match result {
            Ok(v) => {
                let close_enough = if approx {
                    (v - expected).abs() < 1e-3
                } else {
                    (v - expected).abs() < 1e-9
                };
                if !close_enough {
                    failures.push(format!("{:?}: expected {expected}, got {v}", example.expr));
                }
            }
            Err(e) => failures.push(format!(
                "{:?}: expected {expected}, got error {:?}",
                example.expr,
                e.to_string()
            )),
        }
    }

    assert!(
        failures.is_empty(),
        "README.md examples out of sync with the evaluator:\n{}",
        failures.join("\n")
    );
}
