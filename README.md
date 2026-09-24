Expression Calculator (Rust)
=============================

A portable expression calculator, built as a CLI tool (and soon an MCP server) so AI coding agents like Claude and GitHub Copilot can outsource arithmetic instead of computing it themselves — saving tokens and avoiding LLM math mistakes.

Spiritual successor to [excalc](https://github.com/dblock/excalc) (Vestris Inc. Expression Calculator, Pascal, 1996). See [DESIGN.md](DESIGN.md) for the grammar, function catalog, and what's ported vs. deferred vs. skipped.

See [CHANGELOG.md](CHANGELOG.md) for release history and [RELEASING.md](RELEASING.md) for how to cut a new release.

## Usage

```
cargo run -- "2 + 2 * 3"
# 8
```

## Development

```
cargo build
cargo test
```
