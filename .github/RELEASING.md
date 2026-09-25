# Releasing & dependency automation

## Flow

1. **Dependabot** opens grouped PRs every Monday (one for all minor/patch Cargo
   updates, one for major Cargo updates, one for GitHub Actions).
   `dependabot-auto-merge.yml` enables auto-merge (squash) on them, so they merge
   as soon as all required checks pass.
2. **`semver.yml`** runs on every PR to `master`. It merges `master` into the PR
   head and checks whether the published crate is affected:
   - `locale-rs/src/**` changed,
   - `locale-rs/Cargo.toml` changed (other than `version`),
   - the resolved normal/build dependency tree of `locale-rs` changed
     (dev-dependencies and `locale-dev`-only deps are ignored).

   If so, `cargo-semver-checks` determines the required release level against
   `master`, and if the PR's `locale-rs` version is too low the workflow commits
   the bump (`locale-rs/Cargo.toml` + `Cargo.lock`) to the PR branch. For 0.x
   versions, breaking changes bump the minor version, everything else bumps patch.
   A version that is already high enough (e.g. a manual bump) is accepted.
   Fork PRs can't be pushed to; the check fails with the required version instead.
3. **`release.yml`** runs on every push to `master`. If `v<locale-rs version>`
   isn't tagged yet, it creates the tag and calls `publish.yml` to publish to
   crates.io. It then updates open auto-merge PRs that fell behind `master`, so
   their semver check re-runs against the newly released version.

The script behind step 2 can be run locally (requires `cargo-semver-checks`):

```sh
.github/scripts/semver-check.sh origin/master          # report only
.github/scripts/semver-check.sh origin/master --write  # also bump
```

## One-time repository setup

- **`RELEASE_TOKEN` secret**: a fine-grained PAT (or GitHub App token) with
  *Contents: read & write* and *Pull requests: read & write* on this repo.
  Add it both as an **Actions** secret and as a **Dependabot** secret
  (Dependabot-triggered runs can only read Dependabot secrets). Commits pushed
  with `GITHUB_TOKEN` don't trigger CI, so without it the bump can't be pushed.
- **Settings → General → Allow auto-merge**: enabled (squash merging allowed).
- **Branch protection / ruleset on `master`**: require a PR and require the status
  checks (at least `semver`, `build`, `test`, `clippy`, `fmt`,
  `license_check`) to pass. Without required checks, auto-merge would merge
  Dependabot PRs immediately.
- **`CARGO_REGISTRY_TOKEN` secret**: already used by `publish.yml`.
