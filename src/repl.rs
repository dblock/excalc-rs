//! Interactive read-eval-print loop. Variables assigned in one line stay
//! visible to later lines for the rest of the session (unlike a single
//! one-shot [`crate::evaluate_value`] call, which only persists variables
//! within that one call).

use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context as RlContext, Editor, Helper};

use crate::eval::Context;
use crate::functions::arg_hints::ARG_HINTS;
use crate::functions::catalog::FUNCTION_NAMES;

/// What to do with a single line of REPL input.
pub enum LineOutcome {
    /// Nothing to print (e.g. a blank line).
    Silent,
    /// Print this line (a result or an error message), then keep looping.
    Print(String),
    /// Stop the REPL (`exit`, `quit`, or an equivalent command).
    Quit,
}

/// Word boundary used by [`FunctionCompleter`]: everything but letters,
/// digits, `_`, and the trailing `?` used by predicate function names
/// (e.g. `prime?`) breaks a word.
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '?'
}

/// If `pos` sits inside the argument list of a recognized function call
/// (an opening `(` immediately preceded by a name in [`ARG_HINTS`]),
/// returns that lowercased name, the zero-based index of the argument the
/// cursor is currently inside, and the byte offset where that argument's
/// text begins (right after the enclosing `(` or the last top-level `,`
/// at the same nesting depth).
fn call_context(line: &str, pos: usize) -> Option<(String, usize, usize)> {
    let prefix = line.get(..pos)?;
    let mut open_parens: Vec<usize> = Vec::new();
    for (i, c) in prefix.char_indices() {
        match c {
            '(' => open_parens.push(i),
            ')' => {
                open_parens.pop();
            }
            _ => {}
        }
    }
    let open_idx = *open_parens.last()?;
    let before_paren = &prefix[..open_idx];
    let name_start = before_paren
        .rfind(|c: char| !(c.is_alphanumeric() || c == '_'))
        .map(|i| i + 1)
        .unwrap_or(0);
    let name = before_paren[name_start..].to_lowercase();
    if name.is_empty() || !ARG_HINTS.iter().any(|(n, _)| *n == name) {
        return None;
    }

    let mut depth = 0i32;
    let mut arg_index = 0usize;
    let mut segment_start = open_idx + 1;
    for (i, c) in prefix[open_idx + 1..].char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                arg_index += 1;
                segment_start = open_idx + 1 + i + 1;
            }
            _ => {}
        }
    }
    Some((name, arg_index, segment_start))
}

/// Looks up the ghost-text argument name for argument `index` of a known
/// call. `index` is clamped to the last declared argument, so a variadic
/// function's open-ended tail (e.g. `average(a, b, ...)`) keeps hinting
/// its last placeholder for any later argument.
fn arg_hint(name: &str, index: usize) -> Option<&'static str> {
    let args = ARG_HINTS.iter().find(|(n, _)| *n == name)?.1;
    let last = args.len().checked_sub(1)?;
    let effective = index.min(last);
    let raw = args[effective];
    Some(if raw == "..." {
        raw
    } else {
        raw.trim_end_matches("...")
    })
}

/// Tab-completes function names (see [`FUNCTION_NAMES`]) at the cursor, and
/// offers argument-name and closing-paren completions inside a recognized
/// call's argument list (see [`ARG_HINTS`]). Also serves as the REPL's
/// [`Helper`] (with a [`Hinter`] impl for the ghost-text argument hints, and
/// no-op highlighting/validation).
struct FunctionCompleter;

impl Completer for FunctionCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &RlContext<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let start = line[..pos]
            .rfind(|c: char| !is_word_char(c))
            .map(|i| i + 1)
            .unwrap_or(0);
        let word = &line[start..pos];
        if !word.is_empty() {
            let candidates: Vec<Pair> = FUNCTION_NAMES
                .iter()
                .filter(|name| name.starts_with(word))
                .map(|name| Pair {
                    display: name.to_string(),
                    replacement: name.to_string(),
                })
                .collect();
            if !candidates.is_empty() {
                return Ok((start, candidates));
            }
        }

        // Nothing to complete as a function name (e.g. the word is empty,
        // or it's a numeric literal like "16"): Tab advances to the next
        // argument (inserting `, `, the same as typing a comma), or writes
        // the closing `)` once the last argument has been typed. The ghost
        // hint (see `Hinter::hint` below) already shows the argument name
        // and disappears as soon as real text is typed, so we don't insert
        // placeholder text here (it would just get typed over, not replaced).
        let Some((name, index, segment_start)) = call_context(line, pos) else {
            return Ok((start, Vec::new()));
        };
        let Some(args) = ARG_HINTS
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, args)| *args)
        else {
            return Ok((start, Vec::new()));
        };
        let is_last_slot = index >= args.len() - 1;
        let segment_typed = !line[segment_start..pos].trim().is_empty();

        let candidate = if segment_typed && is_last_slot {
            Pair {
                display: ")".to_string(),
                replacement: ")".to_string(),
            }
        } else if !is_last_slot {
            Pair {
                display: ", ".to_string(),
                replacement: ", ".to_string(),
            }
        } else {
            return Ok((start, Vec::new()));
        };
        Ok((pos, vec![candidate]))
    }
}

impl Hinter for FunctionCompleter {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &RlContext<'_>) -> Option<String> {
        let (name, index, segment_start) = call_context(line, pos)?;
        if !line[segment_start..pos].trim().is_empty() {
            return None;
        }
        let hint_text = arg_hint(&name, index)?;
        Some(hint_text.to_string())
    }
}

impl Highlighter for FunctionCompleter {}

impl Validator for FunctionCompleter {}

impl Helper for FunctionCompleter {}

/// Text printed by the `help` REPL command.
const HELP_TEXT: &str = "\
Enter an expression to evaluate it, e.g. 2 + 2 * 3 or sqrt(16) + sin(pi/2).
Assign variables with name := expr; they stay available for later lines.
Press Tab while typing a function name to complete it.
Inside a call's parentheses, argument names are shown as you type; Tab
moves to the next argument or writes the closing `)` once you're done.

Commands:
  help          show this message
  about         show version, author, and license info
  vars          list currently assigned variables
  exit, quit    leave the REPL (Ctrl-D also works)

See https://github.com/dblock/excalc-rs/tree/master/docs for the full function reference.";

/// Text printed by the `about` REPL command.
fn about_text() -> String {
    format!(
        "excalc {}\n{}\nWritten by Daniel (dB.) Doubrovkine <https://code.dblock.org>.\nLicense: {}\nRepository: {}",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_PKG_LICENSE"),
        env!("CARGO_PKG_REPOSITORY"),
    )
}

/// Evaluates a single line of REPL input against a persistent [`Context`].
/// Pure and TTY-free, so it's directly unit-testable without going through
/// rustyline.
pub fn process_line(line: &str, ctx: &mut Context) -> LineOutcome {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return LineOutcome::Silent;
    }
    match trimmed {
        "exit" | "quit" => LineOutcome::Quit,
        "help" => LineOutcome::Print(HELP_TEXT.to_string()),
        "about" => LineOutcome::Print(about_text()),
        "vars" => {
            let mut vars: Vec<(&str, f64)> = ctx.variables().collect();
            if vars.is_empty() {
                LineOutcome::Print("(no variables assigned)".to_string())
            } else {
                vars.sort_by(|a, b| a.0.cmp(b.0));
                let joined = vars
                    .iter()
                    .map(|(name, value)| format!("{name} = {value}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                LineOutcome::Print(joined)
            }
        }
        _ => match crate::evaluate_value_with_context(trimmed, ctx) {
            Ok(value) => LineOutcome::Print(value.to_string()),
            Err(e) => LineOutcome::Print(format!("error: {e}")),
        },
    }
}

/// Runs the interactive REPL against stdin/stdout until the user quits
/// (`exit`, `quit`, or Ctrl-D) or an unrecoverable I/O error occurs. Tab
/// completes function names via [`FunctionCompleter`].
pub fn run() -> rustyline::Result<()> {
    let mut rl: Editor<FunctionCompleter, rustyline::history::DefaultHistory> = Editor::new()?;
    rl.set_helper(Some(FunctionCompleter));
    let mut ctx = Context::new();
    println!(
        "excalc {} — type 'help' for usage, 'exit' or Ctrl-D to quit.",
        env!("CARGO_PKG_VERSION")
    );
    loop {
        match rl.readline("excalc> ") {
            Ok(line) => {
                if !line.trim().is_empty() {
                    let _ = rl.add_history_entry(line.as_str());
                }
                match process_line(&line, &mut ctx) {
                    LineOutcome::Silent => {}
                    LineOutcome::Print(msg) => println!("{msg}"),
                    LineOutcome::Quit => break,
                }
            }
            Err(ReadlineError::Interrupted) => continue, // Ctrl-C: abandon the current line
            Err(ReadlineError::Eof) => break,            // Ctrl-D
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn printed(outcome: LineOutcome) -> String {
        match outcome {
            LineOutcome::Print(s) => s,
            LineOutcome::Silent => String::new(),
            LineOutcome::Quit => "<quit>".to_string(),
        }
    }

    #[test]
    fn evaluates_expressions() {
        let mut ctx = Context::new();
        assert_eq!(printed(process_line("2 + 2", &mut ctx)), "4");
    }

    #[test]
    fn variables_persist_across_lines() {
        let mut ctx = Context::new();
        process_line("x := 5", &mut ctx);
        assert_eq!(printed(process_line("x * 2", &mut ctx)), "10");
    }

    #[test]
    fn blank_line_is_silent() {
        let mut ctx = Context::new();
        assert!(matches!(process_line("", &mut ctx), LineOutcome::Silent));
        assert!(matches!(process_line("   ", &mut ctx), LineOutcome::Silent));
    }

    #[test]
    fn exit_and_quit_stop_the_loop() {
        let mut ctx = Context::new();
        assert!(matches!(process_line("exit", &mut ctx), LineOutcome::Quit));
        assert!(matches!(process_line("quit", &mut ctx), LineOutcome::Quit));
    }

    #[test]
    fn vars_command_lists_assignments() {
        let mut ctx = Context::new();
        assert_eq!(
            printed(process_line("vars", &mut ctx)),
            "(no variables assigned)"
        );
        process_line("x := 5", &mut ctx);
        process_line("y := 10", &mut ctx);
        assert_eq!(printed(process_line("vars", &mut ctx)), "x = 5\ny = 10");
    }

    #[test]
    fn help_command_prints_usage() {
        let mut ctx = Context::new();
        let output = printed(process_line("help", &mut ctx));
        assert!(output.contains("Commands:"));
        assert!(output.contains("vars"));
        assert!(output.contains("exit, quit"));
    }

    #[test]
    fn about_command_prints_author_and_version() {
        let mut ctx = Context::new();
        let output = printed(process_line("about", &mut ctx));
        assert!(output.contains(env!("CARGO_PKG_VERSION")));
        assert!(output.contains("Daniel (dB.) Doubrovkine"));
        assert!(output.contains("License:"));
        assert!(output.contains("github.com/dblock/excalc-rs"));
    }

    #[test]
    fn errors_are_printed_not_panicked() {
        let mut ctx = Context::new();
        assert_eq!(
            printed(process_line("1 / 0", &mut ctx)),
            "error: division by zero"
        );
        assert_eq!(
            printed(process_line("x + 1", &mut ctx)),
            "error: unknown variable: x"
        );
    }

    fn complete(line: &str, pos: usize) -> (usize, Vec<String>) {
        let history = rustyline::history::DefaultHistory::new();
        let rl_ctx = RlContext::new(&history);
        let (start, candidates) = FunctionCompleter
            .complete(line, pos, &rl_ctx)
            .expect("completion should not error");
        (
            start,
            candidates.into_iter().map(|p| p.replacement).collect(),
        )
    }

    #[test]
    fn completes_function_name_prefix() {
        let (start, candidates) = complete("sq", 2);
        assert_eq!(start, 0);
        assert!(candidates.contains(&"sqrt".to_string()));
    }

    #[test]
    fn completes_mid_expression() {
        // Cursor right after "sq" in "2 + sq(9)".
        let (start, candidates) = complete("2 + sq(9)", 6);
        assert_eq!(start, 4);
        assert!(candidates.contains(&"sqrt".to_string()));
    }

    #[test]
    fn no_candidates_for_unknown_prefix() {
        let (_, candidates) = complete("zzz", 3);
        assert!(candidates.is_empty());
    }

    #[test]
    fn empty_word_yields_no_candidates() {
        let (_, candidates) = complete("2 + ", 4);
        assert!(candidates.is_empty());
    }

    fn hint(line: &str, pos: usize) -> Option<String> {
        let history = rustyline::history::DefaultHistory::new();
        let rl_ctx = RlContext::new(&history);
        FunctionCompleter.hint(line, pos, &rl_ctx)
    }

    #[test]
    fn hints_first_argument_after_open_paren() {
        assert_eq!(hint("sqrt(", 5), Some("x".to_string()));
    }

    #[test]
    fn hints_second_argument_after_comma() {
        assert_eq!(hint("atan2(1, ", 9), Some("x".to_string()));
    }

    #[test]
    fn no_hint_once_argument_has_been_typed() {
        assert_eq!(hint("sqrt(1", 6), None);
    }

    #[test]
    fn no_hint_for_grouping_parens() {
        assert_eq!(hint("(1 + 2", 6), None);
    }

    #[test]
    fn hint_ignores_nested_call_commas() {
        // The nested `sqrt(2, 3)` call's inner comma is at depth 1 and must
        // not be mistaken for atan2's own top-level comma; only the comma
        // right after the nested call's `)` should advance atan2's arg index.
        let line = "atan2(sqrt(2, 3), ";
        assert_eq!(hint(line, line.len()), Some("x".to_string()));
    }

    #[test]
    fn tab_advances_past_non_last_empty_argument() {
        // Not the last argument, and nothing typed yet: Tab inserts ", "
        // (same effect as typing a comma) rather than a placeholder, so
        // typing a real value afterwards can't get glued onto ghost text.
        let (start, candidates) = complete("atan2(", 6);
        assert_eq!(start, 6);
        assert_eq!(candidates, vec![", ".to_string()]);
    }

    #[test]
    fn tab_advances_past_non_last_typed_argument() {
        let (start, candidates) = complete("atan2(1", 7);
        assert_eq!(start, 7);
        assert_eq!(candidates, vec![", ".to_string()]);
    }

    #[test]
    fn tab_closes_paren_after_last_argument_typed() {
        let (start, candidates) = complete("sqrt(16", 7);
        assert_eq!(start, 7);
        assert_eq!(candidates, vec![")".to_string()]);
    }

    #[test]
    fn tab_offers_nothing_for_untyped_last_argument() {
        // Nothing typed yet for the last argument: don't offer to close
        // (matches the spec's "when all arguments are entered").
        let (_, candidates) = complete("atan2(1, ", 9);
        assert!(candidates.is_empty());
    }

    #[test]
    fn tab_offers_nothing_for_untyped_only_argument() {
        let (_, candidates) = complete("sqrt(", 5);
        assert!(candidates.is_empty());
    }

    #[test]
    fn variadic_tail_keeps_hinting_and_can_close_anytime() {
        let line = "average(1, 2, ";
        assert_eq!(hint(line, line.len()), Some("...".to_string()));
        let line2 = "average(1, 2, 3";
        let (_, candidates) = complete(line2, line2.len());
        assert_eq!(candidates, vec![")".to_string()]);
    }
}
