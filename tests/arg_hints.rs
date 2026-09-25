//! Guards against `excalc::functions::arg_hints::ARG_HINTS` (used for the
//! REPL's inline function-call hinter) drifting out of sync with the
//! documented function signatures in `docs/functions/*.md`.
//!
//! Every function's row in those tables starts with a Markdown table cell
//! whose first backtick-quoted span is its call signature, e.g.:
//!
//!   | `atan2(y, x)` | ... |
//!   | `ellipticF(k)` / `ellipticF(k, z)` (alias `ellipticK`) | ... |
//!   | `average(a, b, ...)` | `avg` | ... |
//!
//! We re-scrape those signatures here (picking the longest overload per
//! name, e.g. `ellipticF(k, z)` over `ellipticF(k)`) and, for a small set of
//! documented aliases whose own row doesn't repeat the signature, borrow
//! the canonical function's argument names. The result must exactly match
//! `ARG_HINTS`, both directions.

use std::collections::BTreeMap;
use std::fs;

/// Function-name aliases that don't get their own `name(args)` signature in
/// the docs (their row documents the canonical name instead, e.g. `avg` is
/// only mentioned as `` `avg` `` next to `` `average(a, b, ...)` ``).
const ALIASES: &[(&str, &str)] = &[
    ("avg", "average"),
    ("prod", "product"),
    ("fibonacci", "fib"),
    ("eind", "phi"),
    ("elliptick", "ellipticf"),
    ("trapez", "trapezoid"),
    ("trapezoide", "trapezoid"),
    ("ordresix", "ordersix"),
    ("gauss", "int"),
];

/// Returns `true` if `s` looks like a parameter name (`x`, `var`, `a...`)
/// or a bare variadic marker (`...`).
fn is_param_like(s: &str) -> bool {
    if s == "..." {
        return true;
    }
    let base = s.strip_suffix("...").unwrap_or(s);
    !base.is_empty()
        && base.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
        && base.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Parses a `name(arg1, arg2, ...)` call form, returning the lowercased name
/// and its argument list if every argument looks like a parameter name.
fn parse_signature(form: &str) -> Option<(String, Vec<String>)> {
    let open = form.find('(')?;
    let close = form.strip_suffix(')')?.len() + 1;
    if close != form.len() || !form[open + 1..].ends_with(')') {
        return None;
    }
    let name = &form[..open];
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '?')
    {
        return None;
    }
    let inner = &form[open + 1..form.len() - 1];
    let args: Vec<String> = if inner.trim().is_empty() {
        Vec::new()
    } else {
        inner.split(',').map(|a| a.trim().to_string()).collect()
    };
    if !args.iter().all(|a| is_param_like(a)) {
        return None;
    }
    Some((name.to_lowercase(), args))
}

fn scrape_docs() -> BTreeMap<String, Vec<String>> {
    let mut sigs: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/docs/functions");
    for entry in fs::read_dir(dir).expect("failed to read docs/functions") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap_or_else(|_| panic!("failed to read {path:?}"));
        for line in text.lines() {
            let Some(rest) = line.trim_start().strip_prefix("| `") else {
                continue;
            };
            let Some(end) = rest.find('`') else {
                continue;
            };
            let cell = &rest[..end];
            for form in cell.split(" / ") {
                if let Some((name, args)) = parse_signature(form.trim()) {
                    sigs.entry(name)
                        .and_modify(|existing| {
                            if args.len() > existing.len() {
                                *existing = args.clone();
                            }
                        })
                        .or_insert(args);
                }
            }
        }
    }
    for (alias, canonical) in ALIASES {
        if let Some(args) = sigs.get(*canonical).cloned() {
            sigs.insert(alias.to_string(), args);
        }
    }
    sigs
}

#[test]
fn arg_hints_match_documented_signatures() {
    let scraped = scrape_docs();
    let catalog: BTreeMap<String, Vec<String>> = excalc::functions::arg_hints::ARG_HINTS
        .iter()
        .map(|(name, args)| {
            (
                name.to_string(),
                args.iter().map(|a| a.to_string()).collect(),
            )
        })
        .collect();

    // Every catalog function name should have a documented signature.
    for name in excalc::functions::catalog::FUNCTION_NAMES {
        assert!(
            scraped.contains_key(*name),
            "no documented `{name}(...)` signature found in docs/functions/*.md \
             (needed for ARG_HINTS)"
        );
    }

    for (name, args) in &catalog {
        let Some(expected) = scraped.get(name) else {
            panic!(
                "ARG_HINTS has an entry for `{name}` with no matching documented \
                 signature in docs/functions/*.md; remove it from src/functions/arg_hints.rs \
                 or add/fix the alias mapping in tests/arg_hints.rs"
            );
        };
        assert_eq!(
            args, expected,
            "ARG_HINTS entry for `{name}` is {args:?} but docs/functions/*.md \
             documents {expected:?}; update src/functions/arg_hints.rs"
        );
    }

    for name in scraped.keys() {
        assert!(
            catalog.contains_key(name),
            "docs/functions/*.md documents `{name}(...)` but it's missing from \
             ARG_HINTS in src/functions/arg_hints.rs"
        );
    }
}
