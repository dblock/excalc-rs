//! Interactive read-eval-print loop. Variables assigned in one line stay
//! visible to later lines for the rest of the session (unlike a single
//! one-shot [`crate::evaluate_value`] call, which only persists variables
//! within that one call).

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::eval::Context;

/// What to do with a single line of REPL input.
pub enum LineOutcome {
    /// Nothing to print (e.g. a blank line).
    Silent,
    /// Print this line (a result or an error message), then keep looping.
    Print(String),
    /// Stop the REPL (`exit`, `quit`, or an equivalent command).
    Quit,
}

/// Text printed by the `help` REPL command.
const HELP_TEXT: &str = "\
Enter an expression to evaluate it, e.g. 2 + 2 * 3 or sqrt(16) + sin(pi/2).
Assign variables with name := expr; they stay available for later lines.

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
/// (`exit`, `quit`, or Ctrl-D) or an unrecoverable I/O error occurs.
pub fn run() -> rustyline::Result<()> {
    let mut rl = DefaultEditor::new()?;
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
}
