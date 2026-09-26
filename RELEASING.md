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
cargo publish --dry-run
```

Fix anything that fails before proceeding. Do not release on a red build.

## 5. Commit and push

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "Release v0.2.0"
git push origin master
```

Wait for CI on `master` to pass before tagging the release commit.

## 6. Tag and release

```bash
git tag -a v0.2.0 -m "v0.2.0"
git push origin v0.2.0
```

Create the GitHub release after pushing the tag:

```bash
gh release create v0.2.0 --title "v0.2.0" --notes-from-tag
```

Publishing the GitHub release triggers `.github/workflows/release.yml`. A build matrix creates checksummed portable archives for macOS (Apple Silicon and Intel), Linux (ARM64 and x86-64, statically linked with musl), and Windows x86-64. The same workflow publishes the crate to crates.io using Trusted Publishing and attaches the Windows MSI.

One-time setup on crates.io:

1. Open the `excalc` crate's **Settings → Trusted Publishing** page.
2. Add a GitHub Actions publisher for owner `dblock`, repository `excalc-rs`, and workflow `release.yml`.
3. Leave the environment blank; the workflow does not use a GitHub environment.

The workflow needs only the built-in GitHub OIDC permission (`id-token: write`); no GitHub Actions secret or local `cargo login` credential is required.

Publishing is one-way: a version can never be replaced or deleted, only yanked (hidden from new installs, but not removed). If publishing fails after an upload, do not retry with the same version number after changing package contents — bump to the next patch version instead.

## 7. Verify the release workflow

Confirm the release workflow published the crate, portable archives, checksums, and Windows MSI, then updated the Homebrew formula on `master`:

```bash
gh run list --workflow=release.yml --limit 1
gh release view v0.2.0
```

The release should list five platform archives, their `.sha256` files, and an `excalc-<version>-x86_64.msi` asset once the workflow finishes. `cargo search excalc --limit 1` should show the new version, and `Formula/excalc.rb` on `master` should point at the new tag and checksum. Treat a missing asset, missing crate version, stale formula, or red run as a release blocker.

The release job audits, installs, and tests the updated formula before committing it. The commit is pushed with the built-in `GITHUB_TOKEN`, so it does not trigger another set of workflows.

## Notes

- Never force-push tags or rewrite an already-pushed release tag. If a release was cut with a mistake, ship a new patch version instead.
- Homebrew users install via `brew tap dblock/excalc-rs https://github.com/dblock/excalc-rs && brew install excalc` (no `homebrew-` prefix needed since the URL is explicit). The tap lives in this same repo's `Formula/` directory — there is no separate tap repo.
- The MSI installer's WiX definition lives in `wix/main.wxs` (generated once with `cargo wix init`, then committed and hand-maintained). It bundles all three binaries (`excalc`, `calc`, `excalc-mcp`) and an optional "add to PATH" component. `.github/workflows/msi.yml` builds it on every push/PR to catch regressions; `.github/workflows/release.yml` rebuilds it and attaches it to each published GitHub release.
