# crates.io publishing

Both publishing jobs in `.github/workflows/release.yml` request a short-lived
crates.io token with `rust-lang/crates-io-auth-action`, pinned to a commit.
Only those jobs have `id-token: write`. The action revokes its token during
job cleanup (see the [action documentation](https://github.com/rust-lang/crates-io-auth-action)).

Authentication failure is allowed so the publish step can select, in order:

1. `steps.prm_auth.outputs.token`
2. `secrets.CARGO_REGISTRY_TOKEN`
3. `secrets.CARGO_TOKEN`

Keep the legacy secrets as fallbacks. A failed publish still fails the job and
prevents creation of a GitHub release. The fallback handles an unavailable OIDC
token; it does not retry a failed publish with a different token.

## Release prerequisites reviewed

- Automatic releases run only on a push to `main`. Manual instant releases use
  `workflow_dispatch` with `release_mode: instant`.
- Both jobs depend on lint, tests, and the release build. Neither publishes on a
  pull request. Build runs only after lint and all three test platforms pass.
- The automatic job reads changelog fragments, bumps the version, builds, then
  authenticates immediately before publishing. Its authentication and publish
  steps share the same `should_release` guard.
- The manual job preserves its existing version/commit handling and shares the
  `version_committed`/`already_released` guard across build, auth, and publish.
- The patch fragment in `changelog.d/` triggers the next automatic release.
  `Cargo.toml` stays at the current version because PR version edits are rejected
  by the version modification check. Existing fragments also request minor bumps,
  so the aggregate next bump is currently minor (`0.3.0` to `0.4.0`), rather than
  a patch-only release.

## Verification

Run `cargo test --test release_workflow_test` to check both release paths, token
precedence, permissions, ordering, guards, triggers, and CI dependencies without
publishing a package. These checks run with the regular test suite in CI.

On Rust 1.99, the pre-fix code fails
`bash experiments/reproduce-async-trait-lint.sh` with four
`clippy::double_must_use` errors. `async-trait` generates a redundant `must_use`
attribute on async trait methods. The two affected traits have scoped lint
allowances; the same experiment command verifies the fix without disabling
other lint checks.

## Registry migration and live verification

Configure the trusted publisher for the `sync-auth` crate with these caller
workflow values, following the [crates.io instructions](https://crates.io/docs/trusted-publishing):

| Setting | Value |
| --- | --- |
| Repository owner | `link-foundation` |
| Repository | `sync-auth` |
| Workflow filename | `release.yml` |
| Environment | None (the release jobs do not declare one) |

The original migration request calls for rerunning `setup --migrate`. Its tool
is not identified or installed in the issue-solver environment. There is also
no supplied crates.io credential or authenticated registry session to register
the publisher. GitHub access alone does not establish crates.io ownership.

Publisher registration and a real OIDC publication remain **unverified**. A
passing PR workflow checks the release configuration but cannot prove either:
PR events intentionally skip both publishing jobs. After registration, a release
run must show the authentication step succeeded and the publish step actually
uploaded a new crate version. An `already_exists` result or a successful secret
fallback does not verify an OIDC publication. Record that run URL before
considering the migration complete. Keep the fallback secrets, and leave the PR
unmerged while these required checks are blocked.
