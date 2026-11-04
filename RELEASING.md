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

### WinGet submission setup

Stable releases also submit a manifest update for `Vestris.Excalc` to [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs), using [WinGet Releaser](https://github.com/vedantmgoyal9/winget-releaser) and Komac. The job waits for successful crate publishing, archives, and MSI upload, then selects only the `excalc-<version>-x86_64.msi` release asset. It uses the existing accepted package as its manifest base and retains older versions. Prereleases and `workflow_dispatch` artifact rebuilds do not submit WinGet updates.

One-time setup:

1. Keep a `winget-pkgs` fork under the repository owner's account (`dblock/winget-pkgs`), and complete Microsoft's Contributor License Agreement for the submitting account.
2. Open the [prefilled classic token creation form](https://github.com/settings/tokens/new?description=ExCalc%20WinGet&scopes=public_repo,workflow) for that account, choose an expiration, and generate a token with `public_repo` and `workflow` scopes. Token creation requires the browser; `gh` cannot create a classic PAT. WinGet Releaser does not support fine-grained tokens. The `workflow` scope lets Komac synchronize upstream workflow changes into the fork.
3. Store the token as the repository Actions secret `WINGET_TOKEN` with the following command. Paste it at the hidden prompt, never into a workflow or command argument:

```bash
gh secret set WINGET_TOKEN --repo dblock/excalc-rs
```

The built-in `GITHUB_TOKEN` cannot push to the separate fork and submit the upstream PR. A missing `WINGET_TOKEN` fails the job explicitly; if an existing token expires or cannot synchronize the fork, replace it or fix its permissions and rerun the failed job.

## 7. Verify the release workflow

Confirm the release workflow published the crate, portable archives, checksums, and Windows MSI, updated the Homebrew formula on `master`, and submitted the WinGet PR:

```bash
gh run list --workflow=release.yml --limit 1
gh release view v0.2.0
```

The release should list five platform archives, their `.sha256` files, and an `excalc-<version>-x86_64.msi` asset once the workflow finishes. `cargo search excalc --limit 1` should show the new version, and `Formula/excalc.rb` on `master` should point at the new tag and checksum. Treat a missing asset, missing crate version, stale formula, or red run as a release blocker.

The release job audits, installs, and tests the updated formula before committing it. The commit is pushed with the built-in `GITHUB_TOKEN`, so it does not trigger another set of workflows.

For stable releases, the **Submit WinGet update** job records the manifest PR URL in its job summary. Submission is not publication: follow the upstream PR through validation, moderator review, merge, and the publish pipeline. Check the generated manifest schema, MSI version and hash, architecture, machine scope, product codes, and command names.

If submission fails, inspect the logs and check for an existing PR before rerunning the failed job; do not blindly create duplicate submissions. Rerunning failed jobs avoids republishing an already-published crate. An artifact-only `workflow_dispatch` run will not submit a WinGet PR. For manual recovery, use `wingetcreate update Vestris.Excalc --version <version> --urls <published-msi-url>`, review and submit the generated manifests. Once Microsoft's publish pipeline succeeds, run `winget source update`, then `winget show --id Vestris.Excalc --exact` to confirm availability.

## Notes

- The MSI workflow updates templates in `wix/manifests/` and replaces the MSI plus `Vestris.Excalc.manifests.zip` in the `latest-preview` prerelease on master pushes. PRs only upload workflow artifacts. Downloads overlapping publication may need retrying with a fresh manifest archive.
- Build MSI versions use UTC `<year minus 2000>.<month>.<minutes since month start>`; builds within the same minute share a version.
- Never force-push tags or rewrite an already-pushed release tag. If a release was cut with a mistake, publish a new version instead.
- Homebrew users install via `brew tap dblock/excalc-rs https://github.com/dblock/excalc-rs`, `brew trust --formula dblock/excalc-rs/excalc`, then `brew install excalc` (no `homebrew-` prefix needed since the URL is explicit). The tap lives in this same repo's `Formula/` directory — there is no separate tap repo.
- The MSI installer's WiX definition lives in `wix/main.wxs` (generated once with `cargo wix init`, then committed and hand-maintained). It bundles all three binaries (`excalc`, `calc`, `excalc-mcp`) and an optional "add to PATH" component. `.github/workflows/msi.yml` builds it on every push/PR to catch regressions; `.github/workflows/release.yml` rebuilds it and attaches it to each published GitHub release.
