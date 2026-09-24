Expression Calculator (Rust)
=============================

A portable expression calculator, built as a CLI tool (and soon an MCP server) so AI coding agents like Claude and GitHub Copilot can outsource arithmetic instead of computing it themselves — saving tokens and avoiding LLM math mistakes.

Spiritual successor to [excalc](https://github.com/dblock/excalc) (Vestris Inc. Expression Calculator, Pascal, 1996). See [DESIGN.md](DESIGN.md) for the grammar, function catalog, and what's ported vs. deferred vs. skipped, and [docs/](docs/README.md) for detailed per-function reference documentation.

See [CHANGELOG.md](CHANGELOG.md) for release history, [RELEASING.md](RELEASING.md) for how to cut a new release, and [CONTRIBUTING.md](CONTRIBUTING.md) for how to build, test, and contribute.

## Install

```bash
cargo install excalc
```

This installs both the `excalc` and `calc` binaries (identical, `calc` is just a shorter alias) to `~/.cargo/bin` (make sure it's on your `PATH`). Requires a [Rust toolchain](https://rustup.rs/).

If you already have another `calc` on your `PATH`, check `which calc` after installing — `cargo install` won't warn you if it shadows an existing command.

## Usage

```
calc "2 + 2 * 3"
# 8

calc "sqrt(16) + sin(pi/2)"
# 5

calc "1/0"
# error, exit code 1
```

Run `calc --help` (or `excalc --help`) for full usage.

Without installing, you can also run it straight from a checkout of this repo:

```
cargo run -- "2 + 2 * 3"
# 8
```

### Examples

Everything below works today (v1). See [docs/](docs/README.md) for full reference documentation, including domains and error conditions for each function, and what's planned but not yet implemented.

**Arithmetic operators** ([details](docs/functions/operators.md)):

```
calc "2 + 2 * 3"          # 8
calc "10 - 4 / 2"         # 8
calc "7 mod 3"            # 1
calc "2 ^ 10"             # 1024      (power)
calc "2 \ 9"              # 3         (n-th root: the 2nd root of 9)
calc "5!"                 # 120       (factorial)
calc "50%"                # 0.5       (percent)
calc "(2 + 3) * 4"        # 20        (grouping)
calc -- "-5 + 3"          # -2        (unary minus needs -- so clap doesn't treat it as a flag)
```

**Standard math** ([details](docs/functions/standard-math.md)), all angles in radians:

```
calc "sqrt(16)"           # 4
calc "ln(e)"              # 1         (natural log)
calc "log(100)"           # 2         (base 10)
calc "logn(8, 2)"         # 3         (log base 2 of 8)
calc "sin(pi/2)"          # 1
calc "cos(0)"             # 1
calc "tan(pi/4)"          # ~1
calc "asin(1)"            # ~1.5708   (pi/2)
calc "acos(0)"            # ~1.5708   (pi/2)
calc "atan(1)"            # ~0.7854   (pi/4)
calc "sinh(1)"            # ~1.1752
calc "cosh(0)"            # 1
calc "tanh(0)"            # 0
calc "sec(0)"             # 1
calc "csc(pi/2)"          # 1
calc "cot(pi/4)"          # ~1
```

**Statistics** ([details](docs/functions/statistics.md)), variadic unless noted:

```
calc "sum(1, 2, 3, 4)"       # 10
calc "average(2, 4, 6)"      # 4
calc "product(1, 2, 3, 4)"   # 24
calc "min(3, 1, 2)"          # 1
calc "max(3, 1, 2)"          # 3
calc "harmonic(4)"           # ~2.0833
calc "binom(5, 2)"           # 10       (5 choose 2)
```

**Constants**: `pi` and `e` (case-insensitive).

**Errors** exit with status `1` and print a message, e.g.:

```
calc "1/0"                # error: division by zero
calc "sqrt(-1)"           # error: domain error in sqrt
calc "unknownfn(1)"       # error: unknown function: unknownfn
```
