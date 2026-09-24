# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `calc` binary alias for `excalc` (same CLI, shorter name).
- `docs/` reference documentation, per function category, marked v1 (implemented) vs. planned (extracted from the original Pascal source as a porting reference).
- README `### Examples` section covering every v1 operator and function, verified against the evaluator by `tests/readme_examples.rs` so it can't silently drift out of sync.
- `CONTRIBUTING.md` with build/test/PR instructions, replacing the README's `## Development` section.

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
