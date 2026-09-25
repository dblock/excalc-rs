//! Guards against `excalc::functions::catalog::FUNCTIONS` (used for the
//! REPL's tab completion) drifting out of sync with the actual function
//! dispatch code in `src/eval.rs` and `src/functions/integration.rs`.
//!
//! Every match-arm string literal in this codebase that names a callable
//! function is written on a single line together with its `=>`, e.g.:
//!
//!   "average" | "avg" => return stats::average(args),
//!   "trapezoid" | "trapez" | "trapezoide" => Some(Rule::Trapezoid),
//!
//! so we can extract them without a regex dependency: scan each line, and
//! if it contains `=>`, pull out every double-quoted literal that appears
//! before it.

use std::collections::BTreeSet;
use std::fs;

/// Extracts every double-quoted string literal that appears before `=>` on
/// lines that contain `=>`, within the given source text.
fn match_arm_names(source: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for line in source.lines() {
        let Some(arrow) = line.find("=>") else {
            continue;
        };
        let before_arrow = &line[..arrow];
        let mut chars = before_arrow.char_indices().peekable();
        while let Some((i, c)) = chars.next() {
            if c != '"' {
                continue;
            }
            if let Some(end) = before_arrow[i + 1..].find('"') {
                let literal = &before_arrow[i + 1..i + 1 + end];
                // Only treat it as a function name if it looks like one
                // (lowercase alphanumeric/underscore, optionally ending in
                // `?`), which excludes operator-label strings like "or".
                if !literal.is_empty()
                    && literal
                        .trim_end_matches('?')
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                {
                    names.insert(literal.to_string());
                }
                // Skip past this literal's closing quote.
                while let Some((j, _)) = chars.peek() {
                    if *j > i + 1 + end {
                        break;
                    }
                    chars.next();
                }
            }
        }
    }
    names
}

#[test]
fn function_catalog_matches_dispatch_code() {
    // Normalize line endings so this works the same whether the source was
    // checked out with LF or CRLF (e.g. Windows `core.autocrlf`).
    let eval_src = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/eval.rs"))
        .expect("failed to read src/eval.rs")
        .replace("\r\n", "\n");
    // Only scan the body of `call_function`, not the `#[cfg(test)] mod
    // tests` block below it (which also contains string literals, e.g. in
    // assertions, that aren't function names being dispatched).
    let start = eval_src
        .find("fn call_function(")
        .expect("call_function not found in eval.rs");
    let test_mod_start = eval_src
        .find("#[cfg(test)]\nmod tests")
        .expect("test module not found in eval.rs");
    let mut names = match_arm_names(&eval_src[start..test_mod_start]);

    let integration_src = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/functions/integration.rs"
    ))
    .expect("failed to read src/functions/integration.rs")
    .replace("\r\n", "\n");
    let from_name_start = integration_src
        .find("fn from_name(")
        .expect("from_name not found in integration.rs");
    let from_name_end = from_name_start
        + integration_src[from_name_start..]
            .find("\n    }\n")
            .expect("end of from_name not found");
    names.extend(match_arm_names(
        &integration_src[from_name_start..from_name_end],
    ));

    // Handled as special cases directly in `eval::eval` rather than
    // through `call_function`'s match arms.
    for special in ["int", "gauss", "bisect", "secant", "hex", "oct", "bin"] {
        names.insert(special.to_string());
    }

    let catalog: BTreeSet<String> = excalc::functions::catalog::FUNCTIONS
        .iter()
        .map(|(name, _)| name.to_string())
        .collect();

    let missing_from_catalog: Vec<_> = names.difference(&catalog).collect();
    let stale_in_catalog: Vec<_> = catalog.difference(&names).collect();

    assert!(
        missing_from_catalog.is_empty(),
        "functions dispatched in eval.rs/integration.rs but missing from \
         FUNCTIONS (add them to src/functions/catalog.rs): {missing_from_catalog:?}"
    );
    assert!(
        stale_in_catalog.is_empty(),
        "names in FUNCTIONS that no longer appear in eval.rs/integration.rs \
         dispatch code (remove them from src/functions/catalog.rs): {stale_in_catalog:?}"
    );
}
