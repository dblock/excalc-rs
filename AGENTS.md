# Agents

excalc is a Rust library, CLI, and (planned) MCP server implementing a
portable expression calculator — see [DESIGN.md](DESIGN.md) for scope,
grammar, and the function catalog (what's ported, deferred, or skipped).

## Before Committing

Run these and fix anything they report:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

CI runs the same checks on Linux, macOS, and Windows; don't push code that
fails any of them locally.

## Markdown

Do not hard-wrap prose in Markdown files. Write each paragraph as a single
long line and let the reader's editor/viewer soft-wrap it. Only break lines
for actual structure: headings, lists, code blocks, tables.

## Releasing

Follow [RELEASING.md](RELEASING.md) step by step when asked to cut a
release. Update [CHANGELOG.md](CHANGELOG.md) as part of any change that
adds, removes, or changes behavior — add an entry under `[Unreleased]`.

## Porting from the original Pascal engine

When porting more of `common/MCalc.pas` from
[dblock/excalc](https://github.com/dblock/excalc):

- This is a port "in spirit", not line-by-line. Rename things to be
  idiomatic Rust; don't carry over Pascal naming (`MyArcSin`, `TCalcThread`,
  etc.).
- Flag anything that looks like a bug, typo, or dead code in the original
  rather than silently reproducing it — ask before changing behavior that
  isn't clearly a mistake.
- Update the function catalog and scope sections in DESIGN.md as you add
  or intentionally skip functions.
