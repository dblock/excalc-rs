# Agents

excalc is a Rust library, CLI, and MCP server implementing a portable expression calculator — see [DESIGN.md](DESIGN.md) for scope, grammar, and the function catalog (what's ported, deferred, or skipped).

## Before Committing

Run these and fix anything they report:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

CI runs the same checks on Linux, macOS, and Windows; don't push code that fails any of them locally.

## Coverage

CI reports test coverage to [Coveralls](https://coveralls.io/github/dblock/excalc-rs) via `cargo llvm-cov` on every push/PR (see the `coverage` job in `.github/workflows/ci.yml`). No secrets to configure — it authenticates with the built-in `GITHUB_TOKEN`. To check coverage locally:

```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview   # if using rustup
cargo llvm-cov --workspace --all-features
```

When adding new functions/features, add tests that exercise them rather than relying on coverage tooling to catch gaps after the fact. Keep coverage as close to 100% as practical — including error/domain-error branches, not just the happy path — and check `cargo llvm-cov --workspace --all-features --show-missing-lines` before committing if you touched non-trivial logic. Rust stable has no per-line/comment-based coverage exclusion (`#[coverage(off)]` is nightly-only), so if a line is truly unreachable (e.g. a defensive guard for a condition that can't occur with real `f64` inputs), leave it uncovered but add a comment explaining why, rather than writing a contrived test just to hit it.

## Markdown

Do not hard-wrap prose in Markdown files. Write each paragraph as a single long line and let the reader's editor/viewer soft-wrap it. Only break lines for actual structure: headings, lists, code blocks, tables.

## MCP server

`src/bin/excalc-mcp.rs` is a separate binary target gated behind the `mcp` Cargo feature (it pulls in `rmcp`/`tokio`, which aren't needed by the plain CLI). It must build and pass tests both with and without `--features mcp`; don't move MCP-only code into files that compile unconditionally, and don't make `mcp` a default feature. Its integration test (`tests/mcp.rs`) closes stdin (rather than killing the child process) before waiting on exit, so the MCP server shuts down normally and `cargo llvm-cov` can flush its coverage profile — follow the same pattern for any new subprocess-based tests.

## Releasing

Follow [RELEASING.md](RELEASING.md) step by step when asked to cut a release. Update [CHANGELOG.md](CHANGELOG.md) as part of any change that adds, removes, or changes behavior — add an entry under `[Unreleased]`.

## Porting from the original Pascal engine

When porting more of `common/MCalc.pas` from [dblock/excalc](https://github.com/dblock/excalc):

- This is a port "in spirit", not line-by-line. Rename things to be idiomatic Rust; don't carry over Pascal naming (`MyArcSin`, `TCalcThread`, etc.).
- Flag anything that looks like a bug, typo, or dead code in the original rather than silently reproducing it — ask before changing behavior that isn't clearly a mistake.
- Update the function catalog and scope sections in DESIGN.md as you add or intentionally skip functions.

## Documentation

Whenever a function, operator, or CLI behavior is added, changed, or removed:

- Update the relevant page under [docs/](docs/README.md) (add new functions with their domain/examples, move a function from "planned" to "v1" once it's implemented, or fix domains/examples that changed).
- Update the function catalog and index in [DESIGN.md](DESIGN.md) and [docs/README.md](docs/README.md) so every function is listed and linked from somewhere.
- Update the `### Examples` section in [README.md](README.md) so it still demonstrates every currently-implemented (v1) function/operator category, not just a subset. Each example line must follow the format `calc "expr"  # expected` (optionally with a trailing `(explanation)`), since `tests/readme_examples.rs` parses and executes every one of them against the real evaluator — a wrong or missing example fails CI, not just a stale doc.

Keep `docs/` and README examples in sync with what's actually implemented — don't document something as v1 that isn't callable yet, and don't leave a newly-ported function undocumented.
