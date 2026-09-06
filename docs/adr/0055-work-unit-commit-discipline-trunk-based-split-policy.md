# ADR-0055: Work-Unit Commit Discipline — Trunk-Based Split Policy

## Status

Accepted — 2026-09-06

## Context

The cycle `application-stabilization-and-roadmap-convergence` (v0.108.1, 2026-09-06) shipped 13 implementation commits. One of them — `8813556 feat(editor-ready): create wait.ts module and update import paths` — bundled **five distinct work units** into a single commit:

| Sub-task | Description |
| --- | --- |
| B1.1 + B1.2 | New `frontend/src/editor-ready/wait.ts` module + 4 caller rewires |
| D2.2 | `useEditorWorkspaceController` extends to export `.commands` from `commands/catalog.ts` |
| D3.3 | Archcheck rule B9 (`no-direct-scene-mutation`) |
| E.1 | New `deny.toml` (cargo-deny policy for `editor-core`) |
| ADR-0054 + ROADMAP | New ADR + ROADMAP marker update |

This violates the **work-unit-commits** principle: each logical change should land in its own commit so the bisect/revert/blame surface is fine-grained. The bundled commit makes future archaeology harder and hides the blast radius of each sub-task.

### Why we do not retroactively split 8813556

Three constraints make a post-hoc split anti-pattern in this repo:

1. **Trunk-based workflow forbids rebasing `main`.** The repo policy (CONTRIBUTING.md, AGENTS.md) explicitly disallows force-pushes and rebases of `main`. The only split mechanism compatible with this rule is `git revert` + reapply.
2. **Revert + reapply breaks links and doubles history.** A clean split would produce ~15 new commits (1 revert + 5 reapplied sub-tasks + ~9 carry-forward fixes from the cycle recovery). GitHub PR links, blame on lines touched by 8813556, and `git log --follow` traversal of `deny.toml` / `editor-ready/wait.ts` would all lose context.
3. **Production code is already correct.** All 5 sub-tasks compiled, passed archcheck, passed tests, passed `npm run build:check`, and shipped to v0.108.1. The split adds zero functional value.

Additionally, two adjacent carry-forward items from the same cycle are closed by evidence, not by code change:

- **D3.2 — "explicit migration of direct scene-state mutations"**. The verify-report claim of "9 internal call sites that mutate SCENE_DOC/OPERATION_LOG/DIRTY directly" is contradicted by the code: `grep -rn "SCENE_DOC\|OPERATION_LOG\|DIRTY" frontend/src/ --include='*.ts*'` returns only the WASM type declarations in `frontend/src/wasm/editor_application.d.ts` (TypeScript shape, not mutations). All scene state lives behind `frontend/src/scene-session/index.ts` (created in commit `2ce4a1d`). **D3.2 is a no-op — the encapsulation was already correct.**
- **D4.2 — "move workflow code from `editor-bevy/lib.rs` (4257 lines) to `scene_facade.rs`"**. The verify-report based this on the assumption that `scene_facade.rs` lives in `editor-bevy/src/`. The actual location is `editor-wasm/src/scene_facade.rs` (created in D4.1, commit `4b3e14d`) and is re-exported from `editor-wasm/src/lib.rs`. D4.3 (EditorGateway migration to `scene_*` facade, commit `50012b9`) already accomplished the architectural intent. **D4.2 is superseded — D4.3 covered the same goal with the correct crate.**

## Decision

1. **8813556 will not be split retroactively.** It is closed out as **accepted debt (M-4)** with this ADR. Going forward, the trunk-based policy + revert+reapply repulsion makes bundling an unrecoverable mistake — the cost of cleanup exceeds the cost of leaving it bundled.
2. **D3.2 is closed as evidence-based no-op.** No code change required.
3. **D4.2 is closed as superseded by D4.3.** No code change required.
4. **Work-unit discipline becomes a cycle gate.** Future SDDK apply phases must split multi-concern commits into per-sub-task commits before merge. The orchestrator's `sddk-apply` invocations now require an explicit commit-list in the launch packet; bundling >2 sub-tasks into one commit is a verifier BLOCKER.

### Constraints for future apply phases

- **One commit per atomic change.** A commit may touch multiple files if they share a single intent (e.g., new module + its first caller), but should not bundle unrelated concerns (e.g., a new module + an unrelated archcheck rule + a deny policy).
- **Commit message must enumerate sub-tasks when applicable.** If a commit legitimately bundles 2-3 tightly-coupled sub-tasks (e.g., new trait + its blanket impl + its first usage), the commit body MUST list them in `Wave X.Y:` lines.
- **Verify-report will flag any apply commit with >3 sub-tasks** as `M-4: bundled commit` and require an explanatory rationale or a split.

### Recovery-2 outcome (carry-forward from cycle `p-28fce7028ac3c497`)

| Debt | Status | Closing mechanism |
| --- | --- | --- |
| M-4: bundled commit 8813556 | ✅ Accepted (this ADR) | Documented; no split |
| D3.2: explicit migration | ✅ Closed (this ADR) | Evidence-based no-op |
| D4.2: scene_facade move | ✅ Closed (this ADR) | Superseded by D4.3 |
| ~~C-1: archcheck B8~~ | ✅ Closed (recovery-1, commit `0cb2605`) | Clock trait dep injection |

## Consequences

### Positive

- No broken GitHub links or rebased history.
- Future apply phases cannot repeat the bundling mistake.
- Cycle-debt ledger stays accurate and falsifiable.

### Negative

- 8813556 will remain as a 243-line mega-commit in history. `git blame` for any line in those files will point at 8813556 unless the contributor uses `git log --follow <file>` to find the post-recovery cleanup commits.
- This ADR is the **only** durable record of why the bundle exists. If the ADR is deleted, the rationale is lost.

## Alternatives considered

- **Revert + reapply split (~15 commits):** rejected. Trunk-based forbids rebase; revert+reapply breaks links and adds 14 commits for zero functional value.
- **Local git rebase + force-push:** rejected explicitly by repo policy (`AGENTS.md` / CONTEXT.md CI policy).
- **Cherry-pick fix-ups onto 8813556:** not applicable — there is nothing to fix up; the bundled code is correct.

## References

- v0.108.1 cycle: [`docs/specs/application-stabilization-and-roadmap-convergence.md`](../specs/application-stabilization-and-roadmap-convergence.md)
- Bundled commit: `8813556 feat(editor-ready): create wait.ts module and update import paths`
- Recovery-1 (B8 closure): commits `0cb2605` … `5336d7b` (2026-09-06)
- Trunk-based policy: `AGENTS.md` → "Trunk-based workflow: no rebases of main, all merges via stacked PRs"
