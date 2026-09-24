Expression Calculator (Rust)
=============================

A portable expression calculator, built as a CLI tool (and soon an MCP server) so AI coding agents like Claude and GitHub Copilot can outsource arithmetic instead of computing it themselves — saving tokens and avoiding LLM math mistakes.

Spiritual successor to [excalc](https://github.com/dblock/excalc) (Vestris Inc. Expression Calculator, Pascal, 1996). See [DESIGN.md](DESIGN.md) for the grammar, function catalog, and what's ported vs. deferred vs. skipped.

See [CHANGELOG.md](CHANGELOG.md) for release history and [RELEASING.md](RELEASING.md) for how to cut a new release.

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

## Development

```
cargo build
cargo test
```
