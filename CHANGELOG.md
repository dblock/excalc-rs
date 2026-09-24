# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `calc` binary alias for `excalc` (same CLI, shorter name).
- `docs/` reference documentation, per function category, marked v1 (implemented) vs. planned (extracted from the original Pascal source as a porting reference).
- README `### Examples` section covering every v1 operator and function, verified against the evaluator by `tests/readme_examples.rs` so it can't silently drift out of sync.
- `CONTRIBUTING.md` with build/test/PR instructions, replacing the README's `## Development` section.
- CI: test coverage reported to Coveralls via `cargo llvm-cov`, plus a CI status and coverage badge in README.
- Test suite brought to ~99% line coverage: unit tests for every reciprocal/hyperbolic trig function and error path, statistics error paths, evaluator error paths (unknown variable, factorial overflow, log/root domain errors), lexer edge cases (decimals, scientific notation), parser error paths, and a new `tests/cli.rs` exercising the compiled binary end-to-end. The 3 remaining uncovered lines are `tan`/`sec`/`sech` division-by-zero guards that are mathematically unreachable for any real input.
- `excalc-mcp` binary: an MCP server exposing a single `evaluate` tool over stdio (via [rmcp](https://crates.io/crates/rmcp)), so AI agents that speak MCP can call the evaluator directly instead of shelling out to the CLI. Built and installed by default via the `mcp` Cargo feature (opt out with `cargo install excalc --no-default-features` for a CLI-only install). Covered by `tests/mcp.rs`, including a test that keeps the README's example expression in sync with the real evaluator. README documents one-liners to register it with GitHub Copilot CLI (`copilot mcp add`) and Claude Code (`claude mcp add`), plus a worked example combining several functions in one expression.
- General / rounding functions: `abs`, `frac`, `intg` (alias of `trunc`), `round` (round-half-up), `trunc`, `ceil`, `floor`, `random(x)` (uniform random number in `[0, x]`, via the `rand` crate).
- Number theory functions: `gcd`, `lcm`, `fib`, `prime?`, `moebius`, `mersenne`, `perfect`, `fermat`, `safeprime`, `primec`, `primen`, `mersennegen`, `mersgen`, `genmers`, `sigma`, `tau`, `phi`/`eind`, backed by a deterministic Miller-Rabin primality test (valid for the full `u64` range).
- Comparison and logical/bitwise operators: `=`, `>`, `<` (comparison, returning `1`/`0`), `xor`, `xnor`, `and` (`&` synonym), `nand`, `or`, `nor` (bitwise, operating on operands truncated to `i64`), plus `not(x)`, `shl(x, y)`, `shr(x, y)`. Precedence: comparison loosest, then logical-or-family, then logical-and-family, then the existing arithmetic chain. Deviates from the original: `=` now tests equality instead of assigning a variable, and `?` (the original's equality test) was dropped since it's no longer needed.
- Advanced / special functions: `gamma`, `beta`, `pochhammer`, `bth`, `bman`, `ellipticE`, `ellipticF` (alias `ellipticK`), `ellipticCE`, `ellipticCK`, `dilog`, `dawson`, `erf`, `erfc`, `si`, `ssi`, `ci`, `chi`, `fresnelC`, `fresnelS`, `fresnelF`, `fresnelG`. Most of these are thin wrappers around a generic numeric-integration primitive in the original engine (still deferred as a separate future pass), so this port evaluates them with a private adaptive Simpson's-rule integrator as a stand-in; see `docs/functions/advanced.md` for a TODO on revisiting this once general numeric integration lands.

### Fixed

- Removed dead/unreachable `Token::Minus` arm in `parser.rs`'s `parse_primary` (unary minus is always handled earlier by `parse_unary`, so this branch could never execute).
- The original's `Ci`/`Chi` formulas reference an undefined variable `G` (presumably meant to be the Euler-Mascheroni constant), which the generic variable-lookup mechanism silently defaults to `0`, making the original results meaningless. This port hardcodes the actual Euler-Mascheroni constant instead.
- `docs/functions/advanced.md`'s placeholder example `ellipticCK(1) → 5.99158934050168` was wrong; corrected to `1.57079632679490` (`pi/2`), matching `ellipticCE(1)` since both reduce to the same integral at that boundary.

## [0.1.0] - 2026-09-23

### Added

- Initial scaffold: lexer, parser, AST, evaluator.
- Core arithmetic: `+ - * / mod ^ \ (root) ! (factorial) %`, parentheses, unary minus.
- Standard math functions (radians): `sin cos tan asin acos atan sinh cosh tanh asinh acosh atanh sec csc cot asec acsc acot sech csch coth asech acsch acoth sqrt ln log logn`.
- Statistics functions: `sum average product min max harmonic binom`.
- CLI binary (`excalc "2 + 2 * 3"`).
- CI: build/test on Linux, macOS, Windows; `cargo fmt` and `cargo clippy` checks.

[Unreleased]: https://github.com/dblock/excalc-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/dblock/excalc-rs/releases/tag/v0.1.0
