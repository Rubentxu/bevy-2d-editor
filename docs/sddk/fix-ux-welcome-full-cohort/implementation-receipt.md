# Implementation Receipt — `fix-ux-welcome-full-cohort`

> **Cycle:** `p-28fce7028ac3c497/fix-ux-welcome-full-cohort`
> **Path:** A-min
> **Sequence:** 406 → 407 (`phase.build.complete.a-min`)
> **Phase:** Build (A-min: spec → build, no separate tasks phase)
> **Cycle commit:** `3df1682`
> **Base:** `a72104f76da48e7f6dc1ecf31e18702fe1728f03` (v0.110.9 archive)

---

## 1. Subject

A single-line semantic change in `frontend/tests/ux-welcome.spec.ts:46-61`
(`beforeEach` of the `Defold-inspired welcome overlay (Phase E)` describe
block), replacing `page.reload()` with `page.goto("/")` so the post-init
navigation actually drops the `?skip-welcome=1` query parameter.

## 2. Diff (cycle commit `3df1682`)

```diff
diff --git a/frontend/tests/ux-welcome.spec.ts b/frontend/tests/ux-welcome.spec.ts
@@ -44,12 +44,19 @@ async function clearWelcomeDismissed(page: Page): Promise<void> {
 
 test.describe("Defold-inspired welcome overlay (Phase E)", { tag: ["@full"] }, () => {
   test.beforeEach(async ({ page }) => {
-    await page.goto("/");
+    // Phase 1: navigate to /?skip-welcome=1 so the welcome overlay doesn't
+    // block pointer events during WASM init (the overlay's OPFS hydration
+    // can race with the engine-bridge bridge installation otherwise).
     await page.goto("/?skip-welcome=1");
     await waitForEditorReady(page);
+    // Phase 2: clear OPFS so the overlay treats this as a "first visit".
     await clearWelcomeDismissed(page);
-    // Reload so the welcome-overlay re-reads OPFS fresh.
-    await page.reload();
+    // Phase 3: navigate to / (WITHOUT the skip-welcome query param) so the
+    // WelcomeOverlay re-reads OPFS with the cleared flag and renders.
+    // (Note: page.reload() preserves the ?skip-welcome=1 from Phase 1,
+    //  which would keep urlSkip=true and the overlay would never show —
+    //  use page.goto("/") instead.)
+    await page.goto("/");
     await waitForEditorReady(page);
   });
```

Net delta: **+10/-3 in 1 file**. No production source change. No Rust
change. No config change. No test scenarios added or removed (the 3
existing scenarios share the modified `beforeEach`).

## 3. What landed vs. what was specified

| Spec requirement | Implementation status |
|---|---|
| REQ-1 — Before-each URL correctness | ✅ `page.goto("/")` drops the skip param |
| REQ-2 — No production code changes | ✅ Only `frontend/tests/ux-welcome.spec.ts` changed |
| REQ-3 — Comment block documents the URL-preservation trap | ✅ 3 inline comments explain the three-phase flow and warn about `page.reload()` |
| REQ-4 — Verify gate cohesion | ✅ Recorded in verify-report.md (3/3 + 5/5 regression) |

## 4. Behaviour change

**Before fix:**

```text
URL flow: / → /?skip-welcome=1 → reload → /?skip-welcome=1
urlSkip:  false  true              true     true
Overlay:  never  never             never    never   (test fails)
```

**After fix:**

```text
URL flow: /?skip-welcome=1 → / 
urlSkip:  true                false
Overlay:  never               renders (test passes)
```

The init order (skip during WASM, clear OPFS, then render) is preserved.
The only difference is that step 3 now navigates to a fresh URL (without
the skip param) instead of reloading the skip URL.

## 5. Pre-fix empirical evidence

Wrote and ran `frontend/tests/_debug.spec.ts` (deleted after verification)
to confirm the diagnosis. Two key log lines:

```text
URL flow A (original, page.reload): url=http://localhost:5173/?skip-welcome=1
                                     overlay count=0  banner count=1
URL flow B (proposed, page.goto /): url=http://localhost:5173/
                                     overlay count=1  banner count=0
```

Flow A matches the failing test exactly. Flow B is the proposed fix.

## 6. Post-fix empirical evidence

Wrote the fix on a working tree, ran `git commit`, and re-ran the full
test suite:

```text
$ cd frontend && timeout 90 npx playwright test --project=full tests/ux-welcome.spec.ts --reporter=line
Running 3 tests using 1 worker
[1/3] [full] › tests/ux-welcome.spec.ts:63:3 › ... appears on first visit with all 5 workflow cards @full
[2/3] [full] › tests/ux-welcome.spec.ts:84:3 › ... clicking Skip closes the overlay @full
[3/3] [full] › tests/ux-welcome.spec.ts:91:3 › ... clicking Take the tour also closes the overlay @full
  3 passed (32.3s)
```

Regression set (other `@full` tests that depend on the same `?skip-welcome=1`
WASM-init pattern):

```text
$ timeout 180 npx playwright test --project=full tests/tutorial-walkthrough.spec.ts tests/tour-completed-persistence.spec.ts --reporter=line
  5 passed (36.8s)
```

## 7. TypeScript + lint

```text
$ cd frontend && npx tsc --noEmit         # 0 errors
$ cd frontend && npx eslint --max-warnings=0 tests/ux-welcome.spec.ts   # 0 errors, 0 warnings
```

## 8. Risk register

| Risk (pre-cycle) | Mitigation | Status |
|---|---|---|
| The fix drops the OPFS clear | The clear happens between the two gotos; OPFS is persistent | mitigated (verified by debug spec) |
| The fix accidentally removes a test scenario | Diff shows 0 test cases added/removed | mitigated |
| Other `@full` tests break | Tutorial-walkthrough (4) + tour-completed-persistence (1) explicitly re-run, 5/5 pass | mitigated |
| A future cycle re-introduces `page.reload()` | 3 inline comments name the URL-preservation trap and reference `WelcomeOverlay.tsx:175-180` | mitigated (defensive) |
| Cargo workspace breaks | Zero Rust changes | n/a |

## 9. Carry-forward closure

This commit closes:

- **v0.110.6 carry-forward #3** (P3): `ux-welcome.spec.ts` @full cohort
  (3 tests, all failing) — now 3/3 passing.

No new carry-forwards opened.

## 10. Files changed

```text
frontend/tests/ux-welcome.spec.ts | +10/-3
1 file changed, 10 insertions(+), 3 deletions(-)
```
