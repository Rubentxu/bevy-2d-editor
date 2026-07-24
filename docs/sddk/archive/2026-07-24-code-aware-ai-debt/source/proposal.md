# Proposal: Fix code-aware-ai Debt (security gap + UTF-8 panic + FE divergence)

## Intent
The `code-aware-ai` cycle (v0.72.0, ADR-0015) shipped with three HIGH-severity correctness/security bugs and six documentation-drift points. The most severe: the `FORBIDDEN_AI_COMMANDS` filter that both ADR-0015 and ADR-0016 claim is "enforced server-side" is **dead code** — it exists, is unit-tested, but is never called in the request path. An LLM hallucinating a `DeleteSourceFile` command would pass it through unfiltered to the frontend. This proposal closes the three HIGH bugs, two actionable MED items, and corrects all six doc-drift points.

## Scope

### In Scope
- **H1**: Wire `filter_forbidden_commands` into `propose_handler` so forbidden AI commands are rejected server-side (matching ADR claims).
- **H2**: Fix UTF-8 boundary panic in `SourceFilesSource::assemble` and `SceneAssetSource::assemble` (`&content[..actual]` → char-boundary-safe slicing).
- **H3**: Fix frontend `useAIAssistant.ts` to send the budget-filtered `source_files` list (from `assembleMultiSourceContext`) instead of the raw unfiltered array.
- **M1**: Remove dead toggle UI from `ContextDebugSection` (the `onToggleContextSource` handler was never wired in `App.tsx`).
- **M6**: Strengthen `test_priority_order_respected_under_pressure` to assert that low-priority sources are actually dropped/truncated under budget pressure.
- **Doc drift**: Correct ROADMAP.md:221 (✅ DONE v0.72.0), ADR-0015 lines 82-86/104-106/127/138, ADR-0016 lines 72-73.

### Out of Scope
- **M2** (populate `logic_graphs`/`scene_assets` from WASM) — intentionally deferred PR2 scope; needs separate cycle with new WASM calls.
- **M3** (real cache for hot-reload invalidation) — intentionally deferred; no caching layer exists yet.
- **M4** (duplicate `assembleMultiSourceContext` call) — cosmetic waste; fixing requires refactoring the stats flow; defer.
- **M5** (dead `header_overhead` variable) — 1-line cosmetic cleanup; bundle with M4 later.
- **L1-L5** (EDITOR_DOMAIN size, magic number divergence, E2E timing hacks, mock pattern count) — observability/cosmetic; not worth a cycle.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
None. — This is a bugfix/debt-closure cycle. No spec-level requirements change; the existing ADR-0015 requirements (security enforcement, token budget, context composition) are being made to actually hold as documented. No `openspec/specs/` delta needed.

## Approach
Surgical fixes, no architectural changes. Five code touch-points + six doc edits:

1. **H1** — In `propose.rs`, after `parse_tool_calls` returns envelopes, call `filter_forbidden_commands(&envelopes)`. Log rejected commands via `tracing::warn!`. The filter already exists and is tested (`function_calling.rs:105`); this is purely wiring.
2. **H2** — Replace `&content[..actual]` with `content.chars().take(max_chars).collect::<String>()` where `max_chars = actual` (since chars/4 heuristic means `actual` is already a char-based budget, not byte-based). Two sites in `source_impls.rs`.
3. **H3** — In `useAIAssistant.ts`, replace `sourceFiles` (raw) with `assembled.context.source_files` (filtered) in the `fetchPropose` call body.
4. **M1** — Remove the toggle checkbox block from `ContextDebugSection.tsx` and the unused `onToggle`/`disabledSources` props from `AIAssistantPanel.tsx`. Keep the stats display (that works).
5. **M6** — Add assertion in `system_prompt.rs` test: `assert!(!prompt.contains("fn player_health") || budget_was_tight)` — verify the source file content is absent or marked truncated when budget is exhausted.
6. **Doc drift** — Text-only edits to ROADMAP.md + ADR-0015 + ADR-0016.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/ai-proxy/src/handlers/propose.rs` | Modified | Wire `filter_forbidden_commands` call (H1) |
| `crates/ai-proxy/src/context/source_impls.rs` | Modified | Char-boundary-safe slicing at `:94`, `:184` (H2) |
| `crates/ai-proxy/src/context/system_prompt.rs` | Modified | Strengthen priority test at `:314` (M6) |
| `frontend/src/hooks/useAIAssistant.ts` | Modified | Use filtered source_files list (H3) |
| `frontend/src/components/ContextDebugSection.tsx` | Modified | Remove dead toggle UI (M1) |
| `frontend/src/components/AIAssistantPanel.tsx` | Modified | Remove unused toggle props (M1) |
| `docs/ROADMAP.md` | Modified | Line 221: 🔲 Planned → ✅ DONE v0.72.0 |
| `docs/adr/0015-code-aware-ai-context-model.md` | Modified | Correct 4 inaccurate claims |
| `docs/adr/0016-scene-component-authoring.md` | Modified | Correct "server-side enforced" claim |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| H1 wiring breaks E2E test that sends forbidden command | Low | Audit `code-aware-ai.spec.ts` + `scene-component-authoring.spec.ts` first; no test sends forbidden commands (verified in explore) |
| H2 char-based slice changes truncation length vs byte-based | Low | Char slice ≤ byte slice; budget fit improves, never worsens. Update any exact-length assertion. |
| H3 reduced context causes test assertion failure | Low | Test fixtures use small payloads (<5 source files, <1KB each) well under 40KB budget |
| M1 removal breaks ContextDebugSection rendering | Low | Only removes the toggle `<div>`, keeps the stats `<table>` |

## Rollback Plan
All changes are in a feature branch `fix/code-aware-ai-debt`. Revert via `git revert` of the squash-merge commit, or `git checkout main` if pre-merge. No data migrations, no OPFS schema changes, no irreversible state.

## Dependencies
- None. All fixes are self-contained within the existing codebase.

## Success Criteria
- [ ] `filter_forbidden_commands` is called in `propose_handler` (grep confirms call site outside `#[cfg(test)]`)
- [ ] Unit test: sending a `DeleteSourceFile` envelope through `propose_handler` returns it filtered out
- [ ] No `&content[..N]` byte-slicing remains in `source_impls.rs` (grep confirms)
- [ ] `useAIAssistant.ts` sends `assembled.context.source_files` (the filtered list), not raw `sourceFiles`
- [ ] `ContextDebugSection` has no toggle checkboxes (only stats table)
- [ ] `test_priority_order_respected_under_pressure` asserts source file is dropped/truncated
- [ ] ROADMAP.md:221 shows `✅ DONE (v0.72.0, PRs #83 #84 #85)`
- [ ] ADR-0015 + ADR-0016 no longer claim "server-side enforced" for the filter (or the claim is now true)
- [ ] All existing Rust tests pass (583+), all existing Playwright tests pass, `tsc --noEmit` clean, ESLint clean
