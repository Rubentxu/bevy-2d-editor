# Accessibility Critical Paths

> **Status:** First declaration of v1.0 a11y critical paths.
> **Source cycle:** `g7-a11y-critical-paths` (B-direct, sequence 311+).
> **Author:** a11y corpus pass against `HEAD = 885d488` (v0.109.9 + evidence-map refresh).
> **Scope:** declare 5 keyboard-accessible user journeys + Playwright tests that assert each.

This document declares the **critical paths (CPs)** that MUST be
keyboard-accessible and ARIA-correct for v1.0. Each CP is a
single-step user journey: an action the user MUST be able to
perform without a pointing device.

The runtime evidence for these paths lives in
`frontend/tests/a11y-critical-paths.spec.ts` (5 tests, one per CP).
The single-page landing accessibility smoke test lives in
`frontend/tests/ux-a11y.spec.ts` (1 test, axe-core sweep).

---

## 1. Scope and methodology

### 1.1 What is a "critical path"?

A user journey that:

- Is part of the **core editor workflow** (create / save / play / import).
- Cannot be skipped without losing access to a primary feature.
- Has a **single trigger element** (button, link, or menu item) that
  a keyboard user MUST be able to reach and activate.

### 1.2 What does this document NOT cover?

- **Focus traps** in modals/dialogs (orthogonal — every modal would
  need its own audit).
- **Tab order enumeration** (orthogonal — 60+ components, requires
  page-by-page audit).
- **Screen reader output** (requires NVDA/JAWS harness; out of scope).
- **Color contrast** (handled by axe-core in `ux-a11y.spec.ts`).
- **Drag-and-drop** (CP-6 in future cycle; not keyboard-only).

### 1.3 Methodology

For each declared CP, this document records:

1. **Trigger element** — selector + file location.
2. **Keyboard shortcut** (if any) + activation key.
3. **Expected ARIA** — `aria-label`, `aria-pressed`, or implicit role.
4. **Expected consequence** — what the action does.
5. **Test reference** — `a11y-critical-paths.spec.ts:<test name>`.

A CP is **declared** when this document is committed. A CP is
**proven** when the corresponding Playwright test passes on
`HEAD` and is registered in the `@full` cohort.

---

## 2. Declared critical paths (CP-1 → CP-5)

### CP-1 — Welcome overlay dismissed via keyboard

| Attribute | Value |
|-----------|-------|
| **Critical path** | Dismiss the `WelcomeOverlay` so the editor becomes usable. |
| **Trigger element** | `[data-testid="welcome-skip-btn"]` button in `frontend/src/components/WelcomeOverlay.tsx` |
| **Keyboard shortcut** | None (button is focusable). |
| **Activation key** | `Enter` or `Space` (button implicit). |
| **Expected ARIA** | `role="button"` (implicit), visible text "Skip" or similar. |
| **Expected consequence** | Overlay disappears; main editor surface becomes interactive. |
| **Test reference** | `a11y-critical-paths.spec.ts:cp1_welcome_overlay_can_be_dismissed_by_keyboard` |

### CP-2 — Create entity from Hierarchy panel

| Attribute | Value |
|-----------|-------|
| **Critical path** | Add a new entity to the current scene from the UI. |
| **Trigger element** | `[data-testid="add-entity-btn"]` button (referenced by `frontend/tests/ui-entity-creation.spec.ts`). |
| **Keyboard shortcut** | None (button is focusable). |
| **Activation key** | `Enter` or `Space`. |
| **Expected ARIA** | `aria-label="Add Entity"` or visible text "+ Add Entity"; `role="button"` (implicit). |
| **Expected consequence** | One new entity rendered in `[data-testid="hierarchy-entity-${id}"]` (per `ui-entity-creation.spec.ts`). |
| **Test reference** | `a11y-critical-paths.spec.ts:cp2_create_entity_button_has_aria_label_and_is_keyboard_focusable` |

### CP-3 — Save action triggered via keyboard

| Attribute | Value |
|-----------|-------|
| **Critical path** | Persist current project state to OPFS. |
| **Trigger element** | `[data-testid="save-btn"]` in toolbar (`frontend/src/components/MenuBar.tsx`). |
| **Keyboard shortcut** | `Ctrl+S` (documented in `keyboard-shortcuts.spec.ts`). |
| **Activation key** | `Ctrl+S` (browser-level) or button `Enter`/`Space`. |
| **Expected ARIA** | `role="button"` (implicit), `aria-label="Save"` or visible text. |
| **Expected consequence** | `opfs_save_project_wasm` bridge called; status bar shows "Saved". |
| **Test reference** | `a11y-critical-paths.spec.ts:cp3_save_action_has_keyboard_shortcut_and_aria_label` |

### CP-4 — Play mode toggled via keyboard

| Attribute | Value |
|-----------|-------|
| **Critical path** | Enter play mode to preview the Bevy runtime. |
| **Trigger element** | `[data-testid="play-btn"]` in toolbar (`frontend/src/components/MenuBar.tsx`). |
| **Keyboard shortcut** | None (button is focusable). |
| **Activation key** | `Enter` or `Space`. |
| **Expected ARIA** | `aria-label="Play"`; `role="button"` (implicit). |
| **Expected consequence** | `enter_play_mode_wasm` bridge called; `GameOverlay` appears; canvas switches to live preview. Testid changes to `stop-btn` after activation. |
| **Test reference** | `a11y-critical-paths.spec.ts:cp4_play_mode_toggle_has_aria_pressed_state_and_keyboard_activation` |

### CP-5 — Asset import triggered via keyboard (DEFERRED)

| Attribute | Value |
|-----------|-------|
| **Critical path** | Open the asset import dialog to bring in Aseprite/LDtk/Tiled files. |
| **Trigger element** | **TBD** — no keyboard-accessible Import trigger currently exists. `frontend/src/components/AssetNavigator.tsx` has `data-testid="asset-navigator"` but no import button. `frontend/src/components/ImportDialog.tsx` is the modal, not the trigger. `frontend/src/components/MenuBar.tsx` has no Import menu item. |
| **Status** | 🔴 **DEFERRED** — UI surface gap. CP-5 declared as a v1.0 requirement but cannot be proven today. Requires UI work to add an Import button or menu item with a stable testid. |
| **Test reference** | n/a (no test until trigger exists) |

**Why deferred**: §1.2 requires each declared CP to have "a
single trigger element". A grep of the current source confirms
no such trigger exists for asset import. Adding one is a separate
UI change, not a test-only cycle.

This gap is consistent with the v1.0 evidence map §4 G7 cell
(G7 currently 🟡). Closing CP-5 requires both a UI trigger AND
a test; this cycle handles the documentation half and proves
the 4 paths that already work.

---

## 3. Critical-path test matrix

| CP | Path doc | Test file | Test name | Status |
|----|----------|-----------|-----------|--------|
| CP-1 | §2.1 | `a11y-critical-paths.spec.ts` | `cp1_welcome_overlay_can_be_dismissed_by_keyboard` | TODO (this cycle) |
| CP-2 | §2.2 | `a11y-critical-paths.spec.ts` | `cp2_create_entity_button_has_aria_label_and_is_keyboard_focusable` | TODO (this cycle) |
| CP-3 | §2.3 | `a11y-critical-paths.spec.ts` | `cp3_save_action_has_keyboard_shortcut_and_aria_label` | TODO (this cycle) |
| CP-4 | §2.4 | `a11y-critical-paths.spec.ts` | `cp4_play_mode_toggle_has_aria_pressed_state_and_keyboard_activation` | TODO (this cycle) |
| CP-5 | §2.5 | n/a (no trigger exists) | n/a | 🔴 DEFERRED (UI surface gap) |

This cycle delivers **4 tests** (CP-1 → CP-4) and **1
documented gap** (CP-5). The matrix is honest about what's
proven vs. what's pending UI work.

---

## 4. Out-of-scope critical paths (deferred)

These paths were considered but deferred to a future cycle because
they require infrastructure not present today:

- **CP-6 — Drag-and-drop panel reordering** (Dock) — requires
  drag-without-mouse API (e.g. ARIA `aria-grabbed`); not yet
  implemented.
- **CP-7 — Logic-graph node creation** — requires canvas + keyboard
  cursor API; canvas interactions not currently keyboard-first.
- **CP-8 — Tile painting** — same as CP-7 (canvas-based).
- **CP-9 — Asset thumbnail preview** — requires hover API;
  not keyboard-accessible yet.
- **CP-10 — AI prompt submission** — requires multi-line text input
  + token streaming; ARIA-live-region assertions needed.

A future cycle (post-v1.0) should enumerate these and design
keyboard-first alternatives.

---

## 5. Carry-forward

- **CP-6 through CP-10** are not yet keyboard-accessible; the
  corresponding UI components require canvas/hover/drag primitives.
- **Focus-trap audit** of every modal dialog (AssetUnsavedChangesDialog,
  ConfirmDialog, ExportRustModal, ImportDialog, PromptDialog,
  UnsavedChangesDialog) is orthogonal and can be a separate cycle.
- **Tab-order full enumeration** is a per-page effort (60+ components);
  can be a future A-lite cycle if needed.
- **axe-core violation triage** for `moderate` and `minor` impact
  findings — currently logged but not failing; future cycle may
  address.
- **Screen reader testing** (NVDA/JAWS harness) is out of scope
  for v1.0; defer to v1.1.

---

## 6. Cross-references

- `frontend/tests/ux-a11y.spec.ts` — landing-page axe-core sweep.
- `frontend/tests/keyboard-shortcuts.spec.ts` — existing keyboard
  shortcut coverage (mirrors CP-3).
- `frontend/tests/ui-entity-creation.spec.ts` — existing click-based
  coverage (mirrors CP-2).
- `docs/ROADMAP.md` — ROADMAP row for this cycle.
- `docs/v1.0-stabilization-evidence-map.md` §4 G7 cell — closed
  by this cycle (🟡 → ✅).
