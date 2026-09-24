# Releasing

Steps to cut a release of excalc-rs. Follow these exactly, in order.

## 1. Decide the version number

Use [Semantic Versioning](https://semver.org/): `MAJOR.MINOR.PATCH`.

- **PATCH** (`0.1.0` → `0.1.1`): bug fixes, no new functions/behavior.
- **MINOR** (`0.1.0` → `0.2.0`): new functions or features, backward compatible.
- **MAJOR** (`0.1.0` → `1.0.0`): breaking changes (removed/renamed
  functions, changed precedence, changed CLI output format).

Before 1.0.0, breaking changes may also land in a MINOR bump; use
judgment and say so explicitly in the changelog entry.

## 2. Update CHANGELOG.md

- Rename the `[Unreleased]` section to the new version and today's date,
  e.g. `## [0.2.0] - 2026-09-24`.
- Add a fresh empty `[Unreleased]` section above it.
- Update the link references at the bottom of the file:
  - Add `[0.2.0]: https://github.com/dblock/excalc-rs/compare/v0.1.0...v0.2.0`
    (or `.../releases/tag/v0.2.0` if this is the very first release).
  - Update `[Unreleased]` to compare from the new tag to `HEAD`.
- If `[Unreleased]` was empty (nothing to release), stop here — there is
  nothing to release.

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

## 6. Create the GitHub release

```bash
gh release create v0.2.0 --title "v0.2.0" --notes-from-tag
```

Or, to use the changelog entry as the release notes instead of the raw
commit log:

```bash
gh release create v0.2.0 --title "v0.2.0" --notes "$(sed -n '/## \[0.2.0\]/,/## \[/p' CHANGELOG.md | sed '$d')"
```

## 7. Verify CI passed on the release commit

```bash
gh run list --branch master --limit 1
```

Confirm it's green before telling anyone the release is out.

## Notes

- Do not publish to crates.io as part of this process unless explicitly
  asked to — that's a separate, one-way decision the maintainer should
  approve first.
- Never force-push tags or rewrite an already-pushed release tag. If a
  release was cut with a mistake, ship a new patch version instead.
