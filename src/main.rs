use clap::Parser;
use std::io::{IsTerminal, Read};

/// excalc: evaluate a math expression from the command line.
#[derive(Parser)]
#[command(
    name = "excalc",
    version,
    about,
    after_help = "Written by Daniel (dB.) Doubrovkine <https://code.dblock.org>."
)]
struct Cli {
    /// The expression to evaluate, e.g. "2 + 2 * 3" or "sqrt(16) + sin(pi/2)". Reads from stdin if omitted and stdin is not a TTY.
    expression: Vec<String>,

    /// REPL command history file. Defaults to $EXCALC_HISTORY_FILE, then ~/.excalc_history.
    #[arg(long, value_name = "PATH")]
    history_file: Option<String>,

    /// Don't load or save REPL command history across sessions.
    #[arg(long)]
    no_history: bool,
}

fn main() {
    let cli = Cli::parse();
    let mut input = cli.expression.join(" ");
    if input.trim().is_empty() {
        let mut stdin = std::io::stdin();
        if !stdin.is_terminal() {
            let mut buf = String::new();
            if stdin.read_to_string(&mut buf).is_ok() {
                input = buf;
            }
        }
    }
    if input.trim().is_empty() {
        if std::io::stdin().is_terminal() {
            let history_path = if cli.no_history {
                None
            } else {
                excalc::repl::resolve_history_path(cli.history_file.as_deref())
            };
            if let Err(e) = excalc::repl::run(history_path) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
            return;
        }
        eprintln!("usage: excalc <expression>");
        eprintln!();
        eprintln!("Written by Daniel (dB.) Doubrovkine <https://code.dblock.org>.");
        std::process::exit(2);
    }
    match excalc::evaluate_value_formatted(&input) {
        Ok(text) => println!("{text}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
