# Agents

excalc is a Rust library, CLI, and MCP server implementing a portable expression calculator — see [port/DESIGN.md](port/DESIGN.md) for scope, grammar, and the function catalog (what's ported, deferred, or skipped).

## Prerequisites

Before running Cargo commands, follow the platform setup instructions in [CONTRIBUTING.md](CONTRIBUTING.md#prerequisites), including the native linker requirements and the Windows test-runner workaround.

## Before Committing

Run these and fix anything they report:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

When changing Rust code or Dylint configuration, also run both Dylint commands in [CONTRIBUTING.md](CONTRIBUTING.md#dylint) from the repository root. They use the pinned upstream general-purpose lint collection and comment spelling adapter and run in a separate Linux CI job. Fix spelling errors; add legitimate technical terms to the case-sensitive `.spellcheck.txt` vocabulary, not to the upstream dictionary.

CI runs the same checks on Linux, macOS, and Windows; don't push code that fails any of them locally.

## Local CI Claims

[already-tested](https://github.com/dblock/already-tested/tree/v0.1.0) can skip duplicate Test and Lint work on repository-owner-authored PRs. Install the pinned `v0.1.0` CLI using its [local installation instructions](https://github.com/dblock/already-tested/tree/v0.1.0#local-cli) if needed. Other contributors' PRs and all master pushes run normal CI. Coverage, MSI, Homebrew, and releases never use claims.

Run the complete command for a check with its matching OS family, CPU architecture, and Rust stable toolchain. Exact hosted image or OS patch versions do not need to match:

| Workflow | Check ID | Complete command |
| --- | --- | --- |
| `.github/workflows/test.yml` | `test-linux-x86_64`, `test-macos-arm64`, or `test-windows-x86_64` | `cargo build --all-features --verbose && cargo test --all-features --verbose` |
| `.github/workflows/lint.yml` | `lint-linux-x86_64` | `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo clippy --all-targets --no-default-features -- -D warnings` |
| `.github/workflows/lint.yml` | `dylint-linux-x86_64` | `RUSTFLAGS="-D warnings" cargo dylint --all -- --all-targets --all-features && RUSTFLAGS="-D warnings" cargo dylint --all -- --all-targets --no-default-features` |

For Dylint, install the pinned tools and compiler prerequisites from [CONTRIBUTING.md](CONTRIBUTING.md#dylint) first. Do not claim Linux or Windows checks from a macOS run, or claim a check after running only part of its command. Record actual OS, CPU, and toolchain versions; retest after material environment changes that could affect the check.

After validation, commit the tested files and leave the tracked working tree clean. Generate a claim using the matching workflow/check, the complete command you ran, and actual environment details. For example, after a matching macOS ARM64 build and test:

```bash
already-tested claim --workflow .github/workflows/test.yml --check test-macos-arm64 --command 'cargo build --all-features --verbose && cargo test --all-features --verbose' --environment '<actual macOS version, ARM64, rustc and cargo versions>'
```

Paste the entire output into the PR description, preserving its hidden claim. After tracked source or workflow changes, retest, commit, and replace the claim. If a check was not run in a matching environment, omit its claim and let CI run. Claims cover all tracked files and the executing workflow by default; do not narrow file filters. PR-description edits rerun Test and Lint to pick up updated claims.

Claims are trusted statements, not proof that tests ran or that environments match. They cover PR-head source rather than GitHub's synthetic merge tree; keep the independent checks and full master CI.

## Coverage

CI reports test coverage to [Coveralls](https://coveralls.io/github/dblock/excalc-rs) via `cargo llvm-cov` on every push/PR (see `.github/workflows/coverage.yml`). No secrets to configure — it authenticates with the built-in `GITHUB_TOKEN`. To check coverage locally:

```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview   # if using rustup
cargo llvm-cov --workspace --all-features
```

When adding new functions/features, add tests that exercise them rather than relying on coverage tooling to catch gaps after the fact. Keep coverage as close to 100% as practical — including error/domain-error branches, not just the happy path — and check `cargo llvm-cov --workspace --all-features --show-missing-lines` before committing if you touched non-trivial logic. Rust stable has no per-line/comment-based coverage exclusion (`#[coverage(off)]` is nightly-only), so if a line is truly unreachable (e.g. a defensive guard for a condition that can't occur with real `f64` inputs), leave it uncovered but add a comment explaining why, rather than writing a contrived test just to hit it.

## Markdown

Do not hard-wrap prose in Markdown files. Write each paragraph as a single long line and let the reader's editor/viewer soft-wrap it. Only break lines for actual structure: headings, lists, code blocks, tables.

## MCP server

`src/bin/excalc-mcp.rs` is a separate binary target gated behind the `mcp` Cargo feature (it pulls in `rmcp`/`tokio`, which aren't needed by the plain CLI), but `mcp` is a **default** feature so it's built and installed unless someone opts out with `--no-default-features`. It must build and pass tests both with and without that flag. Its integration test (`tests/mcp.rs`) closes stdin (rather than killing the child process) before waiting on exit, so the MCP server shuts down normally and `cargo llvm-cov` can flush its coverage profile — follow the same pattern for any new subprocess-based tests.

## Releasing

Follow [RELEASING.md](RELEASING.md) step by step when asked to cut a release. Update [CHANGELOG.md](CHANGELOG.md) as part of any change that adds, removes, or changes behavior — add an entry under `[Unreleased]`.

Include a linked PR number and contributor GitHub username in each new changelog entry, using `[#123](https://github.com/dblock/excalc-rs/pull/123): Description - [@contributor](https://github.com/contributor).` Once the PR is created, update the entry with its assigned number before considering the PR ready.

## Porting from the original Pascal engine

When porting more of the original engine from [dblock/excalc](https://github.com/dblock/excalc):

- This is a port "in spirit", not line-by-line. Rename things to be idiomatic Rust; don't carry over Pascal naming (`MyArcSin`, `TCalcThread`, etc.).
- Flag anything that looks like a bug, typo, or dead code in the original rather than silently reproducing it — ask before changing behavior that isn't clearly a mistake.
- Update the function catalog and scope sections in port/DESIGN.md as you add or intentionally skip functions.

## Documentation

Whenever a function, operator, or CLI behavior is added, changed, or removed:

- Update the relevant page under [docs/](docs/README.md) with the new functions' domain/examples, or fix domains/examples that changed. Function docs contain explanations only — no ported/planned status; that lives solely in port/DESIGN.md's Function catalog.
- Update the function catalog and scope sections in [port/DESIGN.md](port/DESIGN.md) (the single source of truth for what's ported vs. planned) and the index in [docs/README.md](docs/README.md) so every function is listed and linked from somewhere.
- Update the `### Examples` section in [README.md](README.md) so it still demonstrates every currently-implemented (ported) function/operator category, not just a subset. Each example line must follow the format `calc "expr"  # expected` (optionally with a trailing `(explanation)`), since `tests/readme_examples.rs` parses and executes every one of them against the real evaluator — a wrong or missing example fails CI, not just a stale doc.

Keep `docs/` and README examples in sync with what's actually implemented — don't document something as ported that isn't callable yet, and don't leave a newly-ported function undocumented.
