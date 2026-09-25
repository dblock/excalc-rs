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

/// Tab-completes function names (see [`FUNCTION_NAMES`]) at the cursor.
/// Also serves as the REPL's [`Helper`] (with no-op hinting/highlighting/
/// validation, since we only need completion).
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
        if word.is_empty() {
            return Ok((start, Vec::new()));
        }
        let candidates = FUNCTION_NAMES
            .iter()
            .filter(|name| name.starts_with(word))
            .map(|name| Pair {
                display: name.to_string(),
                replacement: name.to_string(),
            })
            .collect();
        Ok((start, candidates))
    }
}

impl Hinter for FunctionCompleter {
    type Hint = String;
}

impl Highlighter for FunctionCompleter {}

impl Validator for FunctionCompleter {}

impl Helper for FunctionCompleter {}

/// Text printed by the `help` REPL command.
const HELP_TEXT: &str = "\
Enter an expression to evaluate it, e.g. 2 + 2 * 3 or sqrt(16) + sin(pi/2).
Assign variables with name := expr; they stay available for later lines.
Press Tab while typing a function name to complete it.

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
}
