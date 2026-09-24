# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `excalc`/`calc` read the expression from stdin when no argument is given and stdin is not a TTY, e.g. `echo "2 + 2" | calc`.
- `--help` and usage output now credit the author.
- Windows MSI installer: renamed the default install directory from `excalc` to `ExCalc`; added an optional (on by default) "MCP Server Registration" feature that shells out to `copilot mcp add` and `claude mcp add` to register `excalc-mcp` with GitHub Copilot CLI and Claude Code CLI, if either is found on the machine (`src/bin/excalc-mcp-setup.rs`, run as a deferred, impersonated, best-effort custom action).
- New standard math/general functions: `log2`, `cbrt`, `hypot`, `sign`, `clamp`, `lerp`.
- New statistics functions: `median`, `mode`, `variance`, `stddev`, `percentile`, `covariance`, `correlation`.
- New number theory functions: `primorial`, `digitsum`, `digitalroot`, `palindrome?`, `nextprime`.

## [0.2.0] - 2026-09-24

### Added

- Homebrew formula (`Formula/excalc.rb`), installable via `brew tap dblock/excalc-rs https://github.com/dblock/excalc-rs && brew install excalc`. Builds from source via `cargo install`. CI audits/installs/tests the formula on every push/PR via `.github/workflows/homebrew.yml`; `RELEASING.md` documents updating its `url`/`sha256` on every release.
- Windows MSI installer, built with [cargo-wix](https://github.com/volks73/cargo-wix) from `wix/main.wxs` (bundles `excalc`, `calc`, and `excalc-mcp`, with an optional PATH entry). `.github/workflows/msi.yml` builds it on every push/PR; `.github/workflows/release.yml` rebuilds it and attaches it to each published GitHub release.

- `calc` binary alias for `excalc` (same CLI, shorter name).
- `docs/` reference documentation, per function category, marked ported (implemented) vs. planned (extracted from the original Pascal source as a porting reference).
- README `### Examples` section covering every ported operator and function, verified against the evaluator by `tests/readme_examples.rs` so it can't silently drift out of sync.
- `CONTRIBUTING.md` with build/test/PR instructions, replacing the README's `## Development` section.
- CI: test coverage reported to Coveralls via `cargo llvm-cov`, plus a CI status and coverage badge in README.
- Test suite brought to ~99% line coverage: unit tests for every reciprocal/hyperbolic trig function and error path, statistics error paths, evaluator error paths (unknown variable, factorial overflow, log/root domain errors), lexer edge cases (decimals, scientific notation), parser error paths, and a new `tests/cli.rs` exercising the compiled binary end-to-end. The 3 remaining uncovered lines are `tan`/`sec`/`sech` division-by-zero guards that are mathematically unreachable for any real input.
- `excalc-mcp` binary: an MCP server exposing a single `evaluate` tool over stdio (via [rmcp](https://crates.io/crates/rmcp)), so AI agents that speak MCP can call the evaluator directly instead of shelling out to the CLI. Built and installed by default via the `mcp` Cargo feature (opt out with `cargo install excalc --no-default-features` for a CLI-only install). Covered by `tests/mcp.rs`, including a test that keeps the README's example expression in sync with the real evaluator. README documents one-liners to register it with GitHub Copilot CLI (`copilot mcp add`) and Claude Code (`claude mcp add`), plus a worked example combining several functions in one expression.
- General / rounding functions: `abs`, `frac`, `intg` (alias of `trunc`), `round` (round-half-up), `trunc`, `ceil`, `floor`, `random(x)` (uniform random number in `[0, x]`, via the `rand` crate).
- Number theory functions: `gcd`, `lcm`, `fib`, `prime?`, `moebius`, `mersenne`, `perfect`, `fermat`, `safeprime`, `primec`, `primen`, `mersennegen`, `mersgen`, `genmers`, `sigma`, `tau`, `phi`/`eind`, backed by a deterministic Miller-Rabin primality test (valid for the full `u64` range).
- Comparison and logical/bitwise operators: `=`, `>`, `<` (comparison, returning `1`/`0`), `xor`, `xnor`, `and` (`&` synonym), `nand`, `or`, `nor` (bitwise, operating on operands truncated to `i64`), plus `not(x)`, `shl(x, y)`, `shr(x, y)`. Precedence: comparison loosest, then logical-or-family, then logical-and-family, then the existing arithmetic chain. Deviates from the original: `=` now tests equality instead of assigning a variable, and `?` (the original's equality test) was dropped since it's no longer needed.
- Advanced / special functions: `gamma`, `beta`, `pochhammer`, `bth`, `bman`, `ellipticE`, `ellipticF` (alias `ellipticK`), `ellipticCE`, `ellipticCK`, `dilog`, `dawson`, `erf`, `erfc`, `si`, `ssi`, `ci`, `chi`, `fresnelC`, `fresnelS`, `fresnelF`, `fresnelG`. Most of these are thin wrappers around a generic numeric-integration primitive in the original engine; this port routes them through the same shared adaptive-quadrature engine used by the general-purpose numeric integration functions below (`gamma` keeps its own fixed-step Riemann sum instead, to match its pre-existing worked example) — see `docs/functions/advanced.md`.
- Financial functions: `pv`, `fv`, `pmt`, `npv`, `rate`, `cterm`, `term`, `sln`, `syd`, `ddb`, `db`, and the extended payment-timing-aware variants `irate`, `nper`, `paymt`, `fval`, `pval`, `ipaymt`, `ppaymt`. Unlike the advanced/special functions, these are all closed-form (or, for `irate`, a literal secant-method root find) and don't depend on numeric integration.
- General-purpose numeric integration: named composite quadrature rules `trapezoid` (aliases `trapez`/`trapezoide`), `simpson`, `newton`, `boole`, `ordersix` (alias `ordresix`), `weddle` (each over a whole-number sub-interval count `n`), plus adaptive quadrature `int` (alias `gauss`, refining to a requested tolerance). These are the only functions in the port that take an unevaluated expression and a bare variable as arguments instead of plain numbers, evaluating the expression repeatedly at sample points; see `docs/functions/numeric-integration.md`. This closes out every function category originally planned for the port.
- Variables: `name := expr` assigns to a variable, visible to later statements in the same input, which can now be a `;`- or newline-separated sequence of statements (assignments and/or plain expressions); the value of the last statement is the result. Assignment is a statement, not an expression, so it can't be nested or chained (`x := y := 5`), keeping it unambiguous with `=`'s equality meaning. `pi`/`e` remain reserved and can't be reassigned. Variables don't persist across separate `calc` invocations or MCP tool calls.
- Degree/radian support: `deg(x)`/`rad(x)` convert between radians and degrees, and `sind`/`cosd`/`tand`/`asind`/`acosd`/`atand` are degree-native variants of the six basic circular trig functions. Deliberately stateless/explicit rather than reviving the original's global `CalcMode` toggle — see `docs/functions/standard-math.md` and `port/DESIGN.md#angle-units`.

### Fixed

- Removed dead/unreachable `Token::Minus` arm in `parser.rs`'s `parse_primary` (unary minus is always handled earlier by `parse_unary`, so this branch could never execute).
- The original's `Ci`/`Chi` formulas reference an undefined variable `G` (presumably meant to be the Euler-Mascheroni constant), which the generic variable-lookup mechanism silently defaults to `0`, making the original results meaningless. This port hardcodes the actual Euler-Mascheroni constant instead.
- `docs/functions/advanced.md`'s placeholder example `ellipticCK(1) → 5.99158934050168` was wrong; corrected to `1.57079632679490` (`pi/2`), matching `ellipticCE(1)` since both reduce to the same integral at that boundary.
- `docs/functions/financial.md`'s draft named the fixed-declining-balance depreciation function `fdb`; corrected to `db`, matching the name actually dispatched in the original engine.

## [0.1.0] - 2026-09-23

### Added

- Initial scaffold: lexer, parser, AST, evaluator.
- Core arithmetic: `+ - * / mod ^ \ (root) ! (factorial) %`, parentheses, unary minus.
- Standard math functions (radians): `sin cos tan asin acos atan sinh cosh tanh asinh acosh atanh sec csc cot asec acsc acot sech csch coth asech acsch acoth sqrt ln log logn`.
- Statistics functions: `sum average product min max harmonic binom`.
- CLI binary (`excalc "2 + 2 * 3"`).
- CI: build/test on Linux, macOS, Windows; `cargo fmt` and `cargo clippy` checks.

[Unreleased]: https://github.com/dblock/excalc-rs/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/dblock/excalc-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/dblock/excalc-rs/releases/tag/v0.1.0
