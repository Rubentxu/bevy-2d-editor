# G7 — A11y critical paths corpus (specification)

**Cycle**: `p-28fce7028ac3c497/g7-a11y-critical-paths`
**Path**: B-direct
**Phase**: specify (this document serves as spec)
**Date**: 2026-09-08

## Intent

Close the **G7 🟡 gap** (a11y critical paths) by:

1. **Documenting** the 5 user journeys that MUST be keyboard-accessible
   and ARIA-correct: welcome → create entity → save → play mode → asset import.
2. **Adding a Playwright test** (`frontend/tests/a11y-critical-paths.spec.ts`)
   that asserts each journey's keyboard navigation works AND the
   triggering elements have correct ARIA roles.

This is a **dual doc + test cycle**. The doc declares the contract;
the test proves the contract holds for the current implementation.

## Scope

### In scope

**Artifact 1**: `docs/a11y-critical-paths.md` (NEW, ~150 lines)

Sections:
- §1 Scope and methodology (mirror evidence-map §1 structure).
- §2 The 5 declared critical paths (numbered CP-1 → CP-5), each with:
  - Trigger element + selector.
  - Keyboard shortcut (if any).
  - Expected ARIA roles / labels.
  - Reference spec or ADR.
- §3 Critical-path test matrix (what each test asserts).
- §4 Out-of-scope paths (deferred to next cycle).
- §5 Carry-forward.

**Artifact 2**: `frontend/tests/a11y-critical-paths.spec.ts` (NEW, ~200 lines)

5 tests, one per critical path:
- `cp1_welcome_overlay_can_be_dismissed_by_keyboard`
- `cp2_create_entity_button_has_aria_label_and_is_keyboard_focusable`
- `cp3_save_action_has_keyboard_shortcut_and_aria_label`
- `cp4_play_mode_toggle_has_aria_pressed_state_and_keyboard_activation`
- `cp5_asset_import_button_has_aria_label_and_is_keyboard_focusable`

Each test:
1. Loads `/?skip-welcome=1` (or with-welcome for CP-1).
2. Asserts the trigger element has `aria-label` or accessible text.
3. Focuses the element via `page.keyboard.press('Tab')` or programmatic
   focus, asserts `document.activeElement === element`.
4. Activates via keyboard (`Enter` or `Space`), asserts expected
   consequence (overlay dismissed, entity created, save triggered, etc.).

### Out of scope (this cycle)

- Focus-trap audit (would require examining every modal/dialog).
- Tab-order full enumeration (orthogonal — would touch 60+ components).
- Screen reader output assertion (requires NVDA/JAWS harness).
- Color contrast audit (axe-core handles this; deferred to axe cohort).

## Acceptance criteria

- `docs/a11y-critical-paths.md` committed with 5 declared paths.
- `frontend/tests/a11y-critical-paths.spec.ts` registered in
  `playwright.full.config.ts` (or `@a11y` cohort).
- All 5 tests pass on local runner with axe-core 0 violations
  on the test elements.
- No regressions in editor-bevy (414/0/1) or editor-model (313/0/0).
- `npx tsc --noEmit -p .` clean.

## Verification

- `npx playwright test --list` includes the 5 new tests.
- `npx tsc --noEmit -p .` clean.
- `cargo test --workspace` unchanged (no Rust code touched).

## Risk

**Medium risk**: keyboard-focus semantics depend on
`focus({ focusVisible: true })` working in headless Chromium.
If the test element isn't `tabindex`-able, the test will fail.
Mitigation: each test uses `page.locator(...).focus()` first,
then asserts focus received; only then tests Tab navigation.
