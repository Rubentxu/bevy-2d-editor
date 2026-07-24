# Archive Report — `fix/code-aware-ai-debt`

- Change: `fix/code-aware-ai-debt`
- Status: ✅ SHIPPED
- Version: `v0.84.0`
- Type: Bugfix / debt-closure (no new features, no spec changes)
- PR: #120 (single stacked-to-main, squash-merged)
- Merge commit: `c8cf956`
- Tag: `v0.84.0`

## Origin

Post-`v0.83.0` roadmap review discovered `ROADMAP.md:221` showed `code-aware-ai` as 🔲 Planned when it had shipped in v0.72.0. A deep audit of the `code-aware-ai` cycle (ADR-0015) revealed 3 HIGH bugs, 2 MED items, and 6 documentation-drift points — the most severe being a security filter that was dead code despite both ADR-0015 and ADR-0016 claiming "server-side enforced".

## Artifacts archived

- `docs/sddk/archive/2026-07-24-code-aware-ai-debt/source/explore-report.md`
- `docs/sddk/archive/2026-07-24-code-aware-ai-debt/source/proposal.md`

## Code changes shipped

| ID | Severity | Fix | Files |
|----|----------|-----|-------|
| H1 | HIGH (security) | `filter_forbidden_commands` wired into `propose_handler` | `propose.rs`, `openai/mod.rs` |
| H2 | HIGH (panic) | `floor_char_boundary()` helper for UTF-8 safe slicing | `source_impls.rs` (+4 tests) |
| H3 | HIGH (correctness) | `assembleMultiSourceContext` enforces budget on source files | `ai-context.ts` |
| M1 | MED | Removed dead toggle UI from `ContextDebugSection` | `ContextDebugSection.tsx`, `AIAssistantPanel.tsx` |
| M6 | MED | Strengthened priority-order test assertions | `system_prompt.rs` |

## Documentation corrections (6 drift points)

1. `ROADMAP.md:221` — code-aware-ai 🔲 Planned → ✅ DONE v0.72.0
2. `ROADMAP.md:305` — stale "Next: Hito 4 Order 6" → updated to shipped status
3. `ADR-0015:82-86` — "server-side enforced" → corrected (was false, now true after H1 fix)
4. `ADR-0015:104-106` — "invalidate cached context" → clarified as observability-only in v1
5. `ADR-0015:127` — EDITOR_DOMAIN "~1.5k chars" → "~2.9k chars" (at 3k ceiling)
6. `ADR-0016:72-73` — same "server-side enforced" correction

## Verification snapshot

- Rust ai-proxy: 71/71 (57 unit + 14 integration; +4 new multibyte tests + strengthened M6 test)
- TypeScript (`tsc --noEmit`): 0 errors
- ESLint (changed files): 0 warnings
- Vite build: clean. Bundle 346.87 KB gzip (+0.05 KB over v0.83.0)
- Playwright `asset-thumbnails.spec.ts`: 4/4 pass (no regression)
- Playwright `code-aware-ai.spec.ts`: 4/4 fail — **pre-existing** (confirmed identical on baseline; `ai-panel-btn` selector removed in Hito 5 v0.80.0 MenuBar redesign)

## Carried debt (out of scope for this cycle)

- M2: `logic_graphs` / `scene_assets` frontend stubs (intentionally deferred PR2 scope; needs WASM calls)
- M3: real cache for hot-reload invalidation (no caching layer exists)
- M4/M5: duplicate `assembleMultiSourceContext` call + dead `header_overhead` (cosmetic)
- L1: EDITOR_DOMAIN at 2.9k chars (approaching 3k ceiling; needs split/externalize)
- L2-L5: magic number divergence, E2E timing hacks, mock pattern count drift
- `code-aware-ai.spec.ts` 4/4 selector drift (`ai-panel-btn` removed in Hito 5; needs selector update)
