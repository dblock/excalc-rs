# Contributing to excalc-rs

This project is work of [many contributors](https://github.com/dblock/excalc-rs/graphs/contributors).

You're encouraged to submit [pull requests](https://github.com/dblock/excalc-rs/pulls), [propose features and discuss issues](https://github.com/dblock/excalc-rs/issues).

In the examples below, substitute your Github username for `contributor` in URLs.

### Fork the Project

Fork the [project on Github](https://github.com/dblock/excalc-rs) and check out your copy.

```
git clone https://github.com/contributor/excalc-rs.git
cd excalc-rs
git remote add upstream https://github.com/dblock/excalc-rs.git
```

### Build and Test

Ensure that you can build the project and run tests.

```
cargo build
cargo test
```

To also build/test without the [MCP server](README.md#mcp-server) (`excalc-mcp`, on by default; gated behind the `mcp` feature since it pulls in an async runtime):

```
cargo build --no-default-features
cargo test --no-default-features
```

## Contribute Code

### Create a Topic Branch

Make sure your fork is up-to-date and create a topic branch for your feature or bug fix.

```
git checkout master
git pull upstream master
git checkout -b my-feature-branch
```

### Write Tests

Try to write a test that reproduces the problem you're trying to fix or describes a feature that you want to build. Add tests to [tests/](tests) for CLI/integration behavior or as `#[test]` functions alongside the relevant module for unit-level behavior.

We definitely appreciate pull requests that highlight or reproduce a problem, even without a fix.

### Write Code

Implement your feature or bug fix.

Run these and fix anything they report before committing:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

CI runs the same checks on Linux, macOS, and Windows; don't push code that fails any of them locally.

### Write Documentation

Document any new or changed function, operator, or CLI behavior in the relevant page under [docs/](docs/README.md), in [port/DESIGN.md](port/DESIGN.md)'s function catalog, and in the `### Examples` section of [README.md](README.md) (which is checked against the evaluator by `tests/readme_examples.rs`, so it can't drift out of sync). See [AGENTS.md](AGENTS.md#documentation) for details.

### Update Changelog

Add an entry to [CHANGELOG.md](CHANGELOG.md) under `[Unreleased]`, following the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format already used in the file.

### Commit Changes

Make sure git knows your name and email address:

```
git config --global user.name "Your Name"
git config --global user.email "contributor@example.com"
```

Writing good commit logs is important. A commit log should describe what changed and why.

```
git add ...
git commit
```

### Push

```
git push origin my-feature-branch
```

### Make a Pull Request

Go to https://github.com/contributor/excalc-rs and select your feature branch. Click the 'Pull Request' button and fill out the form. Pull requests are usually reviewed within a few days.

### Rebase

If you've been working on a change for a while, rebase with upstream/master.

```
git fetch upstream
git rebase upstream/master
git push origin my-feature-branch -f
```

### Check on Your Pull Request

Go back to your pull request after a few minutes and see whether it passed CI. Everything should look green, otherwise fix issues and amend your commit as described above.

### Be Patient

It's likely that your change will not be merged and that the nitpicky maintainers will ask you to do more, or fix seemingly benign problems. Hang on there!

## Thank You

Please do know that we really appreciate and value your time and work. We love you, really.
