use clap::Parser;

/// excalc: evaluate a math expression from the command line.
#[derive(Parser)]
#[command(
    name = "excalc",
    version,
    about,
    after_help = "Written by Daniel (dB.) Doubrovkine <https://code.dblock.org>."
)]
struct Cli {
    /// The expression to evaluate, e.g. "2 + 2 * 3" or "sqrt(16) + sin(pi/2)"
    expression: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    let input = cli.expression.join(" ");
    if input.trim().is_empty() {
        eprintln!("usage: excalc <expression>");
        eprintln!();
        eprintln!("Written by Daniel (dB.) Doubrovkine <https://code.dblock.org>.");
        std::process::exit(2);
    }
    match excalc::evaluate(&input) {
        Ok(value) => println!("{value}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
