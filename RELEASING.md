# Releasing

Releases use [rust-crate-release](https://github.com/dblock/rust-crate-release), pinned to `v0.1.1` in `.github/workflows/release.yml`. The workflow validates and publishes the crate through crates.io Trusted Publishing, versions the changelog, pushes the release commit and tag, creates the GitHub release, and prepares the next patch version in `Cargo.toml` and `Cargo.lock`. The same run builds the portable archives and MSI, then updates Homebrew. Stable WinGet submissions are a separate manual step.

## One-time setup

1. In GitHub **Settings → Environments**, create `release` and restrict deployments to `master`.
2. On the `excalc` crate's [Settings → Trusted Publishing](https://crates.io/crates/excalc/settings) page, configure a GitHub publisher for owner `dblock`, repository `excalc-rs`, workflow `release.yml`, and environment `release`. Replace the previous publisher with a blank environment; the reusable workflow requires `release`.
3. Ensure repository rules permit ordinary bot pushes to `master` and creation of `vX.Y.Z` tags. A required-PR or required-status-check rule can block release commits even with `contents: write`. Keep force-push and deletion protections.

The workflow uses the built-in `GITHUB_TOKEN` and `id-token: write`; no Cargo-token secret, separate authentication action, or local `cargo login` is required. `actions: read` allows rcr to verify its immutable recovery artifact before publishing.

## 1. Prepare release notes and choose the version

Use [Semantic Versioning](https://semver.org/): `MAJOR.MINOR.PATCH`.

- **PATCH** (`0.1.0` → `0.1.1`): bug fixes, no new functions/behavior.
- **MINOR** (`0.1.0` → `0.2.0`): new functions or features, backward compatible.
- **MAJOR** (`0.1.0` → `1.0.0`): breaking changes (removed/renamed functions, changed precedence, changed CLI output format).

Before 1.0.0, breaking changes may also land in a MINOR bump; use judgment and say so explicitly in the changelog entry. Add meaningful notes under `[Unreleased]` in `CHANGELOG.md`; an empty section is not releasable. Do not date/version the changelog or create a release tag yourself.

The default release version is the one already in `Cargo.toml`, not an automatically incremented version. Adoption advances the previously published `0.3.0` manifest to `0.3.1` so the first default dispatch targets an unpublished version. For a different version, supply the optional workflow `version` input, or update `Cargo.toml`, run `cargo check` to refresh `Cargo.lock`, and commit both.

## 2. Validate and merge

Follow [CONTRIBUTING.md](CONTRIBUTING.md#prerequisites), then run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --no-default-features
```

Merge the changes and unreleased notes onto `master`. Confirm CI is green before publishing. The workflow repeats these checks using `rcr.toml` and also validates the packaged archive before upload.

## 3. Dispatch the release

Select **Actions → Release → Run workflow**, choose `master`, leave `tag` blank, and optionally enter a stable `version`. Equivalently:

```bash
gh workflow run release.yml --repo dblock/excalc-rs --ref master
```

To select a minor release explicitly:

```bash
gh workflow run release.yml --repo dblock/excalc-rs --ref master -f version=0.4.0
```

**This publishes a real release.** For example, releasing `0.3.1` creates `v0.3.1` and then commits `0.3.2` for development. Do not publish, tag, or create the GitHub release manually. Releases created with `GITHUB_TOKEN` do not trigger another release workflow, so artifact and distribution jobs depend on rcr's output in this same run and check out the release tag, not the next-development commit.

Publishing is one-way: a version can never be replaced or deleted, only yanked. A failed run may already have published the crate or pushed its tag. Inspect the run summary, crates.io, remote refs, and [rcr's recovery guidance](https://github.com/dblock/rust-crate-release/blob/v0.1.1/docs/recovery.md) before retrying. Do not blindly dispatch again or change already-published package contents.

## 4. Verify the release workflow

Confirm the release workflow published the crate, portable archives, checksums, and Windows MSI, and updated the Homebrew formula on `master`:

```bash
gh run list --workflow=release.yml --limit 1
gh release view v0.3.1
```

The release should list five platform archives, their `.sha256` files, and an `excalc-<version>-x86_64.msi` asset once the workflow finishes. `cargo search excalc --limit 1` should show the new version, and `Formula/excalc.rb` on `master` should point at the new tag and checksum. Treat a missing asset, missing crate version, stale formula, or red run as a release blocker.

The release job audits, installs, and tests the updated formula before committing it. The commit is pushed with the built-in `GITHUB_TOKEN`, so it does not trigger another set of workflows.

## 5. Submit the WinGet update manually

WinGet submission is intentionally not part of release CI. The previous [WinGet Releaser](https://github.com/vedantmgoyal9/winget-releaser) integration required a classic PAT with `public_repo` and `workflow` scopes; it did not support fine-grained tokens. Organization token policies and SAML authorization requirements can prevent that PAT from accessing `microsoft/winget-pkgs`, even when anonymous access succeeds. Komac can report such API failures as "package does not exist," so that message alone does not mean the package needs an initial submission. The release workflow no longer uses `WINGET_TOKEN`; an existing repository secret can be removed after this change is merged.

1. Check [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) for an existing `Vestris.Excalc` update PR before starting. Use the latest accepted stable manifests as the base, keep a fork such as `dblock/winget-pkgs`, and complete Microsoft's Contributor License Agreement for the submitting account.
2. On Windows, use [WinGetCreate](https://github.com/microsoft/winget-create/blob/main/doc/update.md) to generate an update for the published stable MSI. For example, for the already-published `0.3.2` release:

```powershell
wingetcreate update Vestris.Excalc --version 0.3.2 --urls https://github.com/dblock/excalc-rs/releases/download/v0.3.2/excalc-0.3.2-x86_64.msi --release-notes-url https://github.com/dblock/excalc-rs/releases/tag/v0.3.2 --out .\winget-update
```

Do not pass `--submit` or a token on the command line. This step prepares manifests; submit the PR through the browser using the organization's permitted sign-in flow.

3. Review the generated manifest schema, MSI version and hash, architecture, machine scope, product codes, command names, and locale metadata. Copy the generated version directory into `manifests/v/Vestris/Excalc/0.3.2/` in your fork without removing older versions. Validate it from the fork's root:

```powershell
winget validate --manifest .\manifests\v\Vestris\Excalc\0.3.2
```

4. Commit the reviewed manifests on a topic branch in your fork and open a PR to `microsoft/winget-pkgs` in the browser. Submission is not publication: follow validation, moderator review, merge, and the publish pipeline. Once Microsoft's publish pipeline succeeds, run `winget source update`, then `winget show --id Vestris.Excalc --exact` to confirm availability.

The local templates in `wix/manifests/` are for `latest-preview` assets only; do not submit their placeholder or preview versions as stable manifests. Neither a manual WinGet submission nor retrying a failed submission requires republishing the crate or creating another ExCalc release.

## Artifact-only recovery

To rebuild archives and the MSI for an existing GitHub release without publishing a crate, changing versions/changelog, updating Homebrew, or submitting WinGet, dispatch from `master` with only `tag`:

```bash
gh workflow run release.yml --repo dblock/excalc-rs --ref master -f tag=v0.3.1
```

The workflow verifies the GitHub release exists before building from its tag. Never supply both `tag` and `version`. For a normal release whose downstream jobs failed, prefer **Re-run failed jobs** so successful publishing is not repeated and Homebrew can still complete. WinGet submission remains manual.

## Notes

- The MSI workflow updates templates in `wix/manifests/` and replaces the MSI, `Vestris.Excalc.preview.yaml`, and `Vestris.Excalc.preview.manifests.zip` in the `latest-preview` prerelease on master pushes. PRs only upload workflow artifacts. Downloads overlapping publication may need retrying with a fresh manifest.
- Build MSI versions use UTC `<year minus 2000>.<month>.<minutes since month start>`; builds within the same minute share a version.
- Never force-push tags or rewrite an already-pushed release tag. If a release was cut with a mistake, publish a new version instead.
- Homebrew users install via `brew tap dblock/excalc-rs https://github.com/dblock/excalc-rs`, `brew trust --formula dblock/excalc-rs/excalc`, then `brew install excalc` (no `homebrew-` prefix needed since the URL is explicit). The tap lives in this same repo's `Formula/` directory — there is no separate tap repo.
- The MSI installer's WiX definition lives in `wix/main.wxs` (generated once with `cargo wix init`, then committed and hand-maintained). It bundles all three binaries (`excalc`, `calc`, `excalc-mcp`) and an optional "add to PATH" component. `.github/workflows/msi.yml` builds it on every push/PR to catch regressions; `.github/workflows/release.yml` rebuilds it and attaches it to each published GitHub release.
