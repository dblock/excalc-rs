//! Runs every `excalc "expr"  # expected` example from the `### Examples`
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

/// One `excalc "expr"  # expected` line extracted from README.md.
struct Example {
    expr: String,
    expected: String,
}

fn parse_readme_examples(readme: &str) -> Vec<Example> {
    let mut examples = Vec::new();
    for line in readme.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("excalc ") else {
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
            // README examples inside "..." escape a literal `$` as `\$` so
            // real bash doesn't try to expand it as a positional parameter
            // (`\` isn't otherwise meaningful before `$` in this
            // language); undo that here to get the real expression.
            expr: expr.replace("\\$", "$"),
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

        // A `,`/`_`-grouped and/or `$`/`£`/`€`/`¥`-currency-prefixed expected
        // value (e.g. `1,235`, `250_000`, or `$11`) is compared against the
        // exact formatted (grouping/currency-aware) output rather than a
        // plain numeric comparison.
        if example.expected.contains([',', '_', '$', '£', '€', '¥']) {
            match excalc::evaluate_value_formatted(&example.expr) {
                Ok(text) if text == example.expected => {}
                Ok(text) => failures.push(format!(
                    "{:?}: expected {:?}, got {text:?}",
                    example.expr, example.expected
                )),
                Err(e) => failures.push(format!(
                    "{:?}: expected {:?}, got error {:?}",
                    example.expr,
                    example.expected,
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

/// Checks that within every fenced code block made up of `excalc "expr"  #
/// comment`-style examples, the `#` (and, where every example in the block
/// has one, the trailing `(description)`) starts at the same column, so the
/// block reads as a tidy table rather than ragged text. Examples whose
/// expression spans multiple lines (e.g. a newline-separated `excalc "x :=
/// 5\ny := ..."` example) still count as one row, aligned on their closing
/// line, since that's the line actually holding the `#`.
#[test]
fn readme_examples_hash_comments_are_column_aligned() {
    let readme_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
    let readme = fs::read_to_string(&readme_path).expect("README.md should be readable");

    let mut failures = Vec::new();
    let mut in_fence = false;
    let mut block: Vec<&str> = Vec::new();
    let mut block_start_line = 0;

    for (i, line) in readme.lines().enumerate() {
        if line.trim() == "```" {
            if in_fence {
                check_block_alignment(&block, block_start_line, &mut failures);
                block.clear();
            } else {
                block_start_line = i + 2; // 1-indexed, first line inside the fence
            }
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            block.push(line);
        }
    }

    assert!(
        failures.is_empty(),
        "README.md code blocks have misaligned `#` comments:\n{}",
        failures.join("\n")
    );
}

/// One row of a block: the README line holding the trailing `# comment`
/// (for a multi-line `excalc "..."` statement, this is its closing line,
/// since that's the only line expected to carry a comment) and that line's
/// 1-indexed line number in README.md.
struct Row<'a> {
    line: &'a str,
    lineno: usize,
}

/// Groups a block's lines into one `Row` per example, tracked by counting
/// `"` characters to know when a quoted expression is still open across a
/// line break. Returns `None` if the block isn't made of `excalc` examples
/// at all (e.g. a plain shell/bash snippet).
fn terminal_rows<'a>(block: &[&'a str], block_start_line: usize) -> Option<Vec<Row<'a>>> {
    let mut rows = Vec::new();
    let mut in_string = false;
    let mut saw_any_start = false;
    for (offset, line) in block.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        if !in_string {
            let starts = line.starts_with("excalc \"") || line.starts_with("excalc -- \"");
            if !starts {
                return None; // a non-continuation line that isn't its own example
            }
            saw_any_start = true;
        }
        if line.matches('"').count() % 2 == 1 {
            in_string = !in_string;
        }
        if !in_string {
            rows.push(Row {
                line,
                lineno: block_start_line + offset,
            });
        }
    }
    if !saw_any_start || in_string {
        return None;
    }
    Some(rows)
}

fn check_block_alignment(block: &[&str], block_start_line: usize, failures: &mut Vec<String>) {
    let Some(rows) = terminal_rows(block, block_start_line) else {
        return;
    };
    if rows.is_empty() {
        return;
    }

    let mut hash_column = None;
    for row in &rows {
        // Use the char count (display column), not the byte offset, so
        // multi-byte UTF-8 characters before the `#` (e.g. `£`) don't
        // throw off alignment checks.
        let Some(col) = row.line.chars().position(|c| c == '#') else {
            return; // not every example has a trailing comment; not a table, skip
        };
        match hash_column {
            None => hash_column = Some(col),
            Some(expected_col) if expected_col != col => {
                failures.push(format!(
                    "README.md:{}: `#` at column {col}, expected column {expected_col} \
                     to match the rest of this block: {:?}",
                    row.lineno, row.line
                ));
            }
            _ => {}
        }
    }

    // Only enforce paren alignment if every example in the block has a
    // trailing `(description)`; some rows are bare numbers or `error: ...`
    // with no parenthetical. Search after the `#`, not from the start of
    // the line — the expression itself commonly contains `(`, e.g.
    // `excalc "erfc(1)"  # ... (complementary error function)`.
    let hash_col = hash_column.unwrap();
    let paren_cols: Vec<Option<usize>> = rows
        .iter()
        .map(|row| {
            row.line
                .chars()
                .skip(hash_col)
                .position(|c| c == '(')
                .map(|offset| hash_col + offset)
        })
        .collect();
    if paren_cols.iter().any(Option::is_none) {
        return;
    }
    let mut paren_column = None;
    for (row, col) in rows.iter().zip(paren_cols.into_iter().flatten()) {
        match paren_column {
            None => paren_column = Some(col),
            Some(expected_col) if expected_col != col => {
                failures.push(format!(
                    "README.md:{}: `(` at column {col}, expected column {expected_col} \
                     to match the rest of this block: {:?}",
                    row.lineno, row.line
                ));
            }
            _ => {}
        }
    }
}
