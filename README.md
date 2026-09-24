Expression Calculator (Rust)
=============================

A portable expression calculator, built as a CLI tool (and soon an MCP server) so AI coding agents like Claude and GitHub Copilot can outsource arithmetic instead of computing it themselves — saving tokens and avoiding LLM math mistakes.

Spiritual successor to [excalc](https://github.com/dblock/excalc) (Vestris Inc. Expression Calculator, Pascal, 1996). See [DESIGN.md](DESIGN.md) for the grammar, function catalog, and what's ported vs. deferred vs. skipped.

See [CHANGELOG.md](CHANGELOG.md) for release history and [RELEASING.md](RELEASING.md) for how to cut a new release.

## Install

```bash
cargo install excalc
```

This installs the `excalc` binary to `~/.cargo/bin` (make sure it's on your `PATH`). Requires a [Rust toolchain](https://rustup.rs/).

## Usage

```
excalc "2 + 2 * 3"
# 8

excalc "sqrt(16) + sin(pi/2)"
# 5

excalc "1/0"
# error, exit code 1
```

Run `excalc --help` for full usage.

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
