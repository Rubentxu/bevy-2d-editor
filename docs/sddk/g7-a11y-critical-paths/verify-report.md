# G7 — A11y critical paths corpus (verification report)

**Cycle**: `p-28fce7028ac3c497/g7-a11y-critical-paths`
**Sequence**: 312 → 313
**Phase**: verify

## Light verification (B-direct)

B-direct path uses `phase.verify.complete.b-direct` after build.

## Checks performed

### 1. Diff scope ✅

```
$ git diff --stat HEAD~1 HEAD
docs/a11y-critical-paths.md                                              | 180 +++++++
frontend/playwright.a11y.config.ts                                        |   1 +-
frontend/tests/a11y-critical-paths.spec.ts                                | 129 +++++
docs/sddk/g7-a11y-critical-paths/implementation-receipt.md                | 102 +++++
docs/sddk/g7-a11y-critical-paths/specification.md                         |  84 ++++
5 files changed, 507 insertions(+), 1 deletion(-)
```

All changes scoped to a11y corpus (doc + spec + 4 tests + cohort registration).

### 2. Test ids in cycles ✅

```
$ npx playwright test --list --config=playwright.a11y.config.ts
[chromium] › a11y-critical-paths.spec.ts:49:3 › A11y Critical Paths — CP-1..CP-4 › CP-1
[chromium] › a11y-critical-paths.spec.ts:72:3 › A11y Critical Paths — CP-1..CP-4 › CP-2
[chromium] › a11y-critical-paths.spec.ts:92:3 › A11y Critical Paths — CP-1..CP-4 › CP-3
[chromium] › a11y-critical-paths.spec.ts:111:3 › A11y Critical Paths — CP-1..CP-4 › CP-4
[chromium] › ux-a11y.spec.ts:17:3 › UX Accessibility — Phase 4 › landing axe sweep
Total: 5 tests in 2 files

$ npx playwright test --list --config=playwright.full.config.ts | grep a11y-critical
[chromium] › a11y-critical-paths.spec.ts:49:3 › ... CP-1
[chromium] › a11y-critical-paths.spec.ts:72:3 › ... CP-2
[chromium] › a11y-critical-paths.spec.ts:92:3 › ... CP-3
[chromium] › a11y-critical-paths.spec.ts:111:3 › ... CP-4
```

4 tests registered in BOTH `@a11y` and `@full` cohorts.

### 3. Acceptance criteria ✅

| Criterion | Status |
|-----------|:---:|
| `docs/a11y-critical-paths.md` committed | ✅ |
| 5 critical paths declared (4 proven, 1 deferred) | ✅ |
| 4 Playwright tests registered in @a11y cohort | ✅ |
| 4 Playwright tests registered in @full cohort | ✅ |
| `npx tsc --noEmit -p .` clean | ✅ |
| G7 ready to upgrade 🟡 → ✅ | ✅ |

### 4. No cargo test required ✅

No Rust code changed. Editor-bevy 414/0/1, editor-model 313/0/0 unchanged.

## Result

All light verification checks pass. Cycle ready for `release.complete`.
