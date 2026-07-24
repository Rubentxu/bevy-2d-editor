# Explore Report — `fix/code-aware-ai-debt`

- Change: `fix/code-aware-ai-debt`
- Date: 2026-07-24
- Context quality: C2 (known debt, file:line evidence from audit)
- Triggered by: post-`v0.83.0` roadmap review discovered ROADMAP.md:221 doc-drift; deep audit of `code-aware-ai` (v0.72.0, PRs #83 #84 #85, ADR-0015) revealed security gap + correctness bugs.

## Current State

The `code-aware-ai` feature (Hito 4 Order 6) shipped in v0.72.0 as a 3-PR chain:
- PR #83 — backend `ContextSource` trait + 6 implementations + `FORBIDDEN_AI_COMMANDS`
- PR #84 — frontend `ai-context.ts` orchestrator + token budget
- PR #85 — `ContextDebugSection` UI + 4 E2E tests

ADR-0015 documents the architecture. ADR-0016 (scene-component-authoring) extended it with 3 more commands and 2 more forbidden entries.

**The code compiles and tests pass**, but an audit found that several documented guarantees are **not actually enforced at runtime**. The most severe: the security filter that ADR-0015/0016 claim "enforces server-side" is dead code.

## Affected Areas

### Backend (`crates/ai-proxy/`)
- `src/handlers/propose.rs:143-156` — `propose_handler` calls `parse_tool_calls` directly, never `filter_forbidden_commands`. **H1 — security gap.**
- `src/openai/function_calling.rs:105` — `filter_forbidden_commands` defined + tested but only called from `#[cfg(test)]`. Dead code in production.
- `src/context/source_impls.rs:94` (`SourceFilesSource::assemble`) and `:184` (`SceneAssetSource::assemble`) — `&content[..actual]` byte-slice can panic on UTF-8 boundary. **H2 — runtime panic.**
- `src/context/system_prompt.rs:314-333` — `test_priority_order_respected_under_pressure` only asserts `prompt.contains("Scene Snapshot")`; never asserts source file was dropped. **M6 — weak test.**

### Frontend (`frontend/src/`)
- `services/ai-context.ts:172-173` — `assembleMultiSourceContext` computes `keptSourceFiles` but the consumer (`useAIAssistant.ts:166-179`) sends the full unfiltered list. **H3 — silent FE/backend divergence.**
- `hooks/useAIAssistant.ts:170-171` — `logic_graphs: []` and `scene_assets` hardcoded empty (M2 — deferred, out of scope for this cycle).
- `components/ContextDebugSection.tsx:109-117` — toggle checkboxes rendered but `App.tsx:1362` never passes `onToggleContextSource`. **M1 — dead UI.**

### Documentation
- `docs/ROADMAP.md:221` — `| 6 | code-aware-ai | — | 🔲 Planned |` should be `✅ DONE (v0.72.0, PRs #83 #84 #85)`.
- `docs/adr/0015-code-aware-ai-context-model.md:82-86` — claims "server-side enforced" (false — H1).
- `docs/adr/0016-scene-component-authoring.md:72-73` — same false claim.
- `docs/adr/0015...:104-106` — claims frontend "invalidates cached source-files context" (no cache exists — M3).
- `docs/adr/0015...:127` — "Currently ~1.5k chars" for `EDITOR_DOMAIN` (actual 2933 chars — L1).
- `docs/adr/0015...:138` — "4 new patterns" in mock (actual 7 — L5).

## Approaches

### 1. Wire the filter + fix slicing + fix FE divergence + dead UI + doc drift (RECOMMENDED)
Fix the 3 HIGH + 2 MED items + 6 doc-drift points. No new features, no scope creep.

- **H1**: Call `filter_forbidden_commands(envelopes)` in `propose_handler` after `parse_tool_calls`, log rejected commands. ~5 LOC.
- **H2**: Replace `&content[..actual]` with `content.chars().take(char_count).collect::<String>()` or floor to char boundary. ~4 LOC × 2 sites.
- **H3**: Make `useAIAssistant.ts` use `assembled.context.source_files` (the filtered list) instead of the raw `sourceFiles` array. ~3 LOC.
- **M1**: Either wire `onToggleContextSource` in `App.tsx` or remove the toggle UI from `ContextDebugSection`. Recommend **remove** (simpler; toggle adds state-management complexity for marginal value).
- **M6**: Add `assert!(!prompt.contains("Source Files") || prompt.contains("truncated"))` or assert the source file content is absent. ~3 LOC.
- **Doc drift**: 6 textual corrections across ROADMAP + 2 ADRs.

- Pros: Closes real security gap (H1), prevents panic (H2), aligns FE/backend (H3), removes misleading UI (M1), strengthens test (M6), fixes lying docs. All changes are surgical.
- Cons: None meaningful. Scope is well-bounded.
- Effort: Low (1-2 days).

### 2. Hotfix minimum (H1 + ROADMAP only)
Only wire the security filter + fix ROADMAP line.

- Pros: Fastest (~2 hours).
- Cons: Leaves H2 (panic), H3 (FE divergence), M1 (dead UI), M6 (weak test), and 5 other doc-drift points. The ADRs still lie about 4 other things.
- Effort: Low.

### 3. Full audit closure (HIGH + MED + LOW)
Everything including M2 (populate logic_graphs/scene_assets), M3 (real cache), L1-L5.

- Pros: Complete.
- Cons: M2/M3 were intentionally deferred PR2 scope items. Fixing them opens scope creep (new WASM calls, new caching layer). Better as a separate cycle.
- Effort: Medium (3-5 days).

## Recommendation

**Approach 1.** It closes every HIGH item (security + correctness), the two actionable MED items, and all doc drift. M2/M3 are intentionally deferred and should stay deferred (separate v2 cycle if ever needed). L1-L5 are cosmetic/observability and not worth a cycle on their own.

## Risks

- **H1 fix could break existing tests** if any E2E test sends a forbidden command expecting it to pass. Mitigation: check `code-aware-ai.spec.ts` and `scene-component-authoring.spec.ts` for forbidden-command scenarios before wiring.
- **H2 fix changes truncation behavior** — if any test asserts exact truncated output length, it may need updating. Mitigation: the chars-based slice will be ≤ the byte-based slice, so budget fit improves.
- **H3 fix reduces context sent to LLM** — if a test asserts `source_files.length === N`, it may break when budget truncation kicks in. Mitigation: test fixtures use small payloads well under budget.

## Ready for Proposal

**Yes.** The audit is complete with file:line evidence. All fixes are surgical. No further exploration needed.
