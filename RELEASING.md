# Releasing

Steps to cut a release of excalc-rs. Follow these exactly, in order.

## 1. Decide the version number

Use [Semantic Versioning](https://semver.org/): `MAJOR.MINOR.PATCH`.

- **PATCH** (`0.1.0` → `0.1.1`): bug fixes, no new functions/behavior.
- **MINOR** (`0.1.0` → `0.2.0`): new functions or features, backward compatible.
- **MAJOR** (`0.1.0` → `1.0.0`): breaking changes (removed/renamed functions, changed precedence, changed CLI output format).

Before 1.0.0, breaking changes may also land in a MINOR bump; use judgment and say so explicitly in the changelog entry.

## 2. Update CHANGELOG.md

- Rename the `[Unreleased]` section to the new version and today's date, e.g. `## [0.2.0] - 2026-09-24`.
- Add a fresh empty `[Unreleased]` section above it.
- Update the link references at the bottom of the file:
  - Add `[0.2.0]: https://github.com/dblock/excalc-rs/compare/v0.1.0...v0.2.0` (or `.../releases/tag/v0.2.0` if this is the very first release).
  - Update `[Unreleased]` to compare from the new tag to `HEAD`.
- If `[Unreleased]` was empty (nothing to release), stop here — there is nothing to release.

## 3. Bump the version in Cargo.toml

Update `version = "..."` in `Cargo.toml` to match. Then run:

```bash
cargo build
```

This regenerates `Cargo.lock` with the new version. Commit both files.

## 4. Verify everything passes locally

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Fix anything that fails before proceeding. Do not release on a red build.

## 5. Commit, tag, and push

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "Release v0.2.0"
git tag -a v0.2.0 -m "v0.2.0"
git push origin master
git push origin v0.2.0
```

## 6. Publish to crates.io

Requires, one-time, on the maintainer's machine:

- A crates.io account with a **verified email address** (crates.io → Account Settings → Profile → set and verify email). `cargo publish` fails with a 400 error until this is done.
- An API token with the "publish-new" scope (crates.io → Account Settings → API Tokens → New Token), logged in locally with `cargo login` (it reads the token from stdin — do not pass it as a command-line argument).

Check whether login has already happened with `cat ~/.cargo/credentials.toml`. If there's no token configured, stop and ask the maintainer to run `cargo login` themselves — don't attempt to work around missing credentials.

```bash
cargo publish --dry-run   # sanity check packaging first
cargo publish
```

Publishing is one-way: a version can never be replaced or deleted, only yanked (hidden from new installs, but not removed). If `cargo publish` fails partway, do not retry with the same version number after fixing the issue — bump to the next patch version instead.

## 7. Create the GitHub release

```bash
gh release create v0.2.0 --title "v0.2.0" --notes-from-tag
```

Or, to use the changelog entry as the release notes instead of the raw commit log:

```bash
gh release create v0.2.0 --title "v0.2.0" --notes "$(sed -n '/## \[0.2.0\]/,/## \[/p' CHANGELOG.md | sed '$d')"
```

## 8. Update the Homebrew formula

Update `Formula/excalc.rb`'s `url` and `sha256` to point at the new tag:

```bash
curl -sL https://github.com/dblock/excalc-rs/archive/refs/tags/v0.2.0.tar.gz -o /tmp/excalc.tar.gz
shasum -a 256 /tmp/excalc.tar.gz
```

Update the `url` to `.../refs/tags/v0.2.0.tar.gz` and `sha256` to the value printed above. Then verify it locally before committing:

```bash
brew tap dblock/excalc-rs "$(pwd)"
brew audit --strict dblock/excalc-rs/excalc
brew install --build-from-source dblock/excalc-rs/excalc
brew test dblock/excalc-rs/excalc
brew uninstall excalc && brew untap dblock/excalc-rs
```

Commit and push the formula update:

```bash
git add Formula/excalc.rb
git commit -m "Update Homebrew formula to v0.2.0"
git push origin master
```

CI also runs this same audit/install/test on every push via `.github/workflows/homebrew.yml` — treat a red run there as a blocker, same as any other CI failure.

## 9. Verify the MSI installer was attached to the release

Publishing the release (step 7) triggers `.github/workflows/release.yml`, which builds a Windows MSI installer with `cargo wix` and uploads it as a release asset automatically. Confirm it showed up:

```bash
gh run list --workflow=release.yml --limit 1
gh release view v0.2.0
```

The release should list an `excalc-<version>-x86_64.msi` asset once the workflow finishes. Treat a missing asset or a red run the same as any other CI failure.

## 10. Verify CI passed on the release commit

```bash
gh run list --branch master --limit 1
```

Confirm it's green before telling anyone the release is out.

## Notes

- Never force-push tags or rewrite an already-pushed release tag. If a release was cut with a mistake, ship a new patch version instead.
- Homebrew users install via `brew tap dblock/excalc-rs https://github.com/dblock/excalc-rs && brew install excalc` (no `homebrew-` prefix needed since the URL is explicit). The tap lives in this same repo's `Formula/` directory — there is no separate tap repo.
- The MSI installer's WiX definition lives in `wix/main.wxs` (generated once with `cargo wix init`, then committed and hand-maintained). It bundles all three binaries (`excalc`, `calc`, `excalc-mcp`) and an optional "add to PATH" component. `.github/workflows/msi.yml` builds it on every push/PR to catch regressions; `.github/workflows/release.yml` rebuilds it and attaches it to the GitHub release when one is published.
