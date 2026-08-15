# Branch Protection

This document lists the five required CI checks that the repository administrator must enable in GitHub Settings to enforce the quality gates on the `main` branch.

## Path

**GitHub Settings → Branches → Branch protection rules → main** (or create a new rule targeting `main`).

## Required Checks

Enable all five checks by ticking **Require status checks to pass before merging** and selecting each check from the status check list:

| # | Check name (as shown in CI) | Purpose |
|---|---|---|
| 1 | `Rust workspace` | Runs `cargo fmt --check`, `cargo test --workspace --all-targets --release --locked`, and `cargo check -p editor-core --target wasm32-unknown-unknown --locked` |
| 2 | `Frontend static gates` | Runs `npm run lint`, `npm run format:check`, and `npx tsc --noEmit` |
| 3 | `Frontend build and bundle budget` | Runs `npm run build:check` (verifies bundle size against ADR-0029 budget) |
| 4 | `Documentation drift check` | Runs `tools/docs-check/check.ts` to enforce ROADMAP/CHANGELOG/ADR table hygiene |
| 5 | `Architecture fitness` | Runs `tools/archcheck/check.ts` to enforce dependency-direction, size budgets, and typed-boundary assertions |

## Notes

- All five checks must be **required** (not optional) on `main` before merging any pull request.
- The `Architecture fitness` check ships as a skeleton in PR1 (v0.87); its assertion set grows in subsequent PRs.
- If a new check is added to `.github/workflows/ci.yml`, update this document and enable the new check in GitHub Settings.
- Branch protection rules can be configured via GitHub UI at `https://github.com/<owner>/<repo>/settings/branches` or via `gh api` / Terraform for automation.
