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
   crates.io. It then refreshes open PRs that fell behind `master`, so their
   semver check re-runs against the newly released version:
   - Dependabot PRs get `@dependabot rebase` (or `@dependabot recreate` if other
     commits, such as a version bump, are on the branch). This also resolves
     `Cargo.lock` conflicts between Dependabot PRs.
   - CLDR bump PRs are regenerated from `master` by re-running `cldr-bump`.
   - Other PRs with auto-merge enabled are updated via "Update branch".
4. **`changelog.yml`** runs after a successful publish. It generates release notes
   from the merged PRs (categories in `.github/release.yml`: breaking, CLDR,
   dependencies, other), creates the GitHub release and opens an auto-merging PR
   that adds the entry to `CHANGELOG.md`. PRs that need a major/breaking release
   are labelled `breaking` by the semver check. It can also be run manually
   (Actions → Changelog → Run workflow) for an existing tag.

The script behind step 2 can be run locally (requires `cargo-semver-checks`):

```sh
.github/scripts/semver-check.sh origin/master          # report only
.github/scripts/semver-check.sh origin/master --write  # also bump
```

## One-time repository setup

- **Bot GitHub App**: commits and PRs created with `GITHUB_TOKEN` don't trigger
  CI, so the automation (version bumps, CLDR and changelog PRs, PR refreshes
  and `@dependabot` commands) acts as a dedicated GitHub App. Its own identity
  also lets a ruleset allow only the bot to push to `cldr-bump/*`.
  1. Avatar → *Settings* → *Developer settings* → *GitHub Apps* → *New GitHub
     App*. Any name and homepage URL, webhook *Active* off, installable *Only on
     this account*.
  2. Repository permissions, all **Read and write**: *Actions*, *Contents*,
     *Pull requests*, *Workflows*.
  3. Create it, note the **Client ID**, *Generate a private key* (downloads a
     `.pem`), then *Install App* on this repository only.
  4. Repo → *Settings* → *Secrets and variables* → **Actions**: add
     `BOT_CLIENT_ID` (the Client ID) and `BOT_PRIVATE_KEY` (the `.pem` contents).
  5. Same two secrets under *Secrets and variables* → **Dependabot** (runs
     triggered by Dependabot can only read Dependabot secrets).

  A `RELEASE_TOKEN` secret (personal access token) is still used as a fallback
  when the app secrets are missing.
- **Ruleset for `cldr-bump/**`** (optional): target pattern `cldr-bump/**`,
  rules *Restrict creations* and *Restrict updates*, bypass list: only the bot
  app. Leave deletions unrestricted so merged branches can be cleaned up.
- **Settings → General → Allow auto-merge**: enabled (squash merging allowed).
- **Branch protection / ruleset on `master`**: enable *Require branches to be
  up to date before merging*, so a PR can't merge with a version that was
  checked against an outdated `master` (two PRs releasing the same version).
  Also require a PR and require the status checks `semver`, `test`, `clippy`,
  `fmt`, `cargo-deny (licenses)` and `build (<os>, stable)` for all three OSes.
  Without required checks, auto-merge would merge Dependabot PRs immediately.
- **`CARGO_REGISTRY_TOKEN` secret**: already used by `publish.yml`.
