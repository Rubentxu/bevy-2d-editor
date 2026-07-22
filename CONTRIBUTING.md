# Contributing

Thank you for improving Bevy 2D Editor. Keep changes focused, tested, and aligned with the domain language in [CONTEXT.md](CONTEXT.md) and the roadmap in [docs/ROADMAP.md](docs/ROADMAP.md).

## Setup

Prerequisites are a stable Rust toolchain, Node.js 22 or newer, npm, `just`, Chromium dependencies, and Git.

```sh
git clone https://github.com/Rubentxu/bevy-2d-editor.git
cd bevy-2d-editor
just setup
just test-install
just dev
```

## Trunk-based workflow

This repository uses **trunk-based development**. The `main` branch is always green and deployable.

### Principles

1. **`main` is sacred**: every commit on `main` must build and pass all tests.
2. **Short-lived branches**: feature branches live hours to a few days, never weeks.
3. **Small PRs**: aim for 200-400 lines of diff. If larger, split by phase (use the SDDK task units).
4. **One concern per PR**: don't mix a feature, a refactor, and a fix.
5. **Merge to main via PR**: no direct pushes to `main`.
6. **Tag-driven releases**: `vX.Y.Z` tag cuts a release; tags trigger CI release workflow.

### Branching model

| Pattern | When | Lifespan |
|---|---|---|
| `feat/<short-name>` | New capability | Hours-days, squash-merged |
| `fix/<short-name>` | Bug fix | Hours-days, squash-merged |
| `chore/<short-name>` | Tooling, deps, config | Hours-days, squash-merged |
| `docs/<short-name>` | Documentation only | Hours-days, squash-merged |
| `release/vX.Y.Z` | Release prep (optional) | Days |
| `hotfix/vX.Y.Z-patch` | Urgent patch on a release tag | Hours |

**Forbidden patterns** (do not create):
- Long-lived personal branches (`my-wip`, `experiment-xyz`)
- Version-prefixed long-lived branches (`v2-features`)
- Branches with embedded PR numbers (`feat-x-pr3`)
- Branches without a topic (`patch`, `fix-bug`)

### Workflow steps

```sh
# 1. Start from latest main
git checkout main
git pull --rebase origin main

# 2. Create a short-lived branch
git checkout -b feat/command-palette-recent

# 3. Work in small commits (or squash later)
git commit -m "feat(ux): add recent commands to palette"
git commit -m "test(ux): cover recent command insertion"

# 4. Run quality gates locally
just test
cd frontend && npm run lint && npm run format:check

# 5. Push and open a PR (use --set-upstream on first push)
git push --set-upstream origin feat/command-palette-recent
# Open PR at https://github.com/Rubentxu/bevy-2d-editor/compare/main...feat/command-palette-recent

# 6. After CI green + review approval, squash-merge to main
# (use the "Squash and merge" button on GitHub; this preserves a linear history on main)

# 7. Delete the branch
git push origin --delete feat/command-palette-recent
```

### When to use stacked branches

Stacked branches are **allowed only** when:
- A single feature is too large for one PR (e.g., a 1500+ LOC cycle).
- The phases have a hard dependency (Phase B can't compile without Phase A).
- The SDDK tasks.md declares an explicit split.

In those cases:
1. Create `feat/<name>-pr1`, `feat/<name>-pr2`, etc.
2. PR2 branches off PR1's head.
3. Once PR1 merges, rebase PR2 onto `main` (which now contains PR1).
4. Squash-merge each independently.

### Releases

Releases are tag-driven:

```sh
# After main is green and the team agrees to release
git checkout main
git pull --rebase origin main
git tag -a v0.80.0 -m "v0.80.0 — Defold-inspired layout + UX overhaul"
git push origin v0.80.0
```

The CI release workflow (`.github/workflows/release.yml`) builds the release WASM bundle and attaches it as a GitHub Release artifact.

## Commit messages

Use **Conventional Commits** with an optional scope:

```text
feat(editor): add scene instance placement
fix(opfs): await catalog metadata persistence
test(a11y): cover command palette focus
docs(roadmap): record v0.78.0
chore(deps): bump vite to 6.5
```

- Imperative mood, ≤ 72 chars in subject.
- Body explains the **why**, not the **what** (the diff shows what).
- Reference ADR numbers when relevant (`ADR-0019`).

When squash-merging, write a single Conventional Commit message in the PR body — GitHub uses it as the final commit message.

## Quality gates

Before opening a pull request, run the relevant commands:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace --all-targets --release --locked
cd frontend && npm run lint
cd frontend && npm run format:check
cd frontend && npm run build:check
just test                  # full Playwright suite
```

The CI Rust lint job is temporarily advisory because existing warnings are noisy; new code should still avoid adding warnings.

## SDDK workflow (non-trivial capabilities)

Multi-PR capabilities (≥ 400 LOC or cross-cutting) follow the project SDDK phases: explore, propose, spec, design, tasks, apply, verify, archive. Refer to the existing artifacts under `sddk/` and architecture decisions under `docs/adr/`. Start implementation only after the relevant specification and design are agreed; update an ADR when a durable architectural decision changes.

Smaller changes (≤ 400 LOC, single concern) can skip the SDDK ceremony and go straight to a single PR.

## Pull request checklist

- [ ] The change is scoped and follows the approved SDD/SDDK artifacts where applicable.
- [ ] Branch name matches the pattern (`feat/`, `fix/`, `chore/`, `docs/`).
- [ ] PR is ≤ 400 LOC (or split with explicit justification).
- [ ] Rust tests and affected Playwright tests pass locally AND in CI.
- [ ] Frontend lint, formatting, production build, and bundle budget pass.
- [ ] Rust formatting passes; new Clippy warnings were not introduced.
- [ ] Accessibility behavior is covered when UI semantics or interaction change.
- [ ] User documentation and [CHANGELOG.md](CHANGELOG.md) are updated for user-visible changes.
- [ ] ADRs and [docs/adr/README.md](docs/adr/README.md) are updated when an architectural decision changes.
- [ ] No generated build output, credentials, or local OPFS data is committed.
- [ ] Branch is ≤ 7 days old (otherwise split or close).

## Code of conduct

A standalone Code of Conduct has not yet been adopted. Until one is added, participate respectfully, assume good intent, and report unacceptable conduct privately to the repository maintainers through the GitHub repository owner.

## Security reports

Do not open a public issue for a suspected vulnerability. Follow [SECURITY.md](SECURITY.md).
