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

## SDD/SDDK workflow

Non-trivial capabilities follow the project SDDK phases: explore, propose, spec, design, tasks, apply, verify, and archive. Refer to the existing artifacts under [docs/sddk](docs/sddk) and architecture decisions under [docs/adr](docs/adr). Start implementation only after the relevant specification and design are agreed; update an ADR when a durable architectural decision changes.

## Branching model

The repository uses a stacked-to-main model:

1. Branch from the latest `main` for the smallest independently reviewable slice.
2. For a multi-PR change, branch the next slice from the preceding slice and clearly state the dependency.
3. Keep each PR green and rebase or retarget the stack as earlier slices merge.
4. Do not mix unrelated refactors with a feature or fix.

Suggested branch names are `feat/<change>-pr1`, `fix/<change>`, `test/<change>`, or `docs/<change>`.

## Commit messages

Use Conventional Commits with an optional scope:

```text
feat(editor): add scene instance placement
fix(opfs): await catalog metadata persistence
test(a11y): cover command palette focus
docs(roadmap): record v0.78.0
```

Use the imperative mood, keep the subject concise, and explain motivation or migration details in the body when they are not obvious.

## Quality gates

Before opening a pull request, run the relevant commands:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace --all-targets --release --locked
cd frontend && npm run lint
cd frontend && npm run format:check
cd frontend && npm run build:check
just test
```

The CI Rust lint job is temporarily advisory because existing warnings are noisy; new code should still avoid adding warnings.

## Pull request checklist

- [ ] The change is scoped and follows the approved SDD/SDDK artifacts where applicable.
- [ ] Rust tests and affected Playwright tests pass.
- [ ] Frontend lint, formatting, production build, and bundle budget pass.
- [ ] Rust formatting passes; new Clippy warnings were not introduced.
- [ ] Accessibility behavior is covered when UI semantics or interaction change.
- [ ] User documentation and [CHANGELOG.md](CHANGELOG.md) are updated for user-visible changes.
- [ ] ADRs and [docs/adr/README.md](docs/adr/README.md) are updated when an architectural decision changes.
- [ ] No generated build output, credentials, or local OPFS data is committed.

## Code of conduct

A standalone Code of Conduct has not yet been adopted. Until one is added, participate respectfully, assume good intent, and report unacceptable conduct privately to the repository maintainers through the GitHub repository owner.

## Security reports

Do not open a public issue for a suspected vulnerability. Follow [SECURITY.md](SECURITY.md).
