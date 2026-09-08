# Accessibility Critical Paths

> **Status:** Updated to reflect CP-5 keyboard-accessible import trigger (v0.110.4 cycle `cp5-import-trigger`).
> **Source cycle:** `g7-a11y-critical-paths` (B-direct, sequence 311+) + `cp5-import-trigger` (B-direct).
> **Author:** a11y corpus pass against `HEAD = c6d383e` (v0.109.9 + G8 archive); CP-5 promoted in `cp5-import-trigger`.
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

### CP-5 — Asset import triggered via keyboard (✅ PROVEN in `cp5-import-trigger`)

| Attribute | Value |
|-----------|-------|
| **Critical path** | Open the asset import dialog (file picker entry point) to bring in Aseprite/LDtk/Tiled files. |
| **Trigger element** | `[data-testid="import-asset-btn"]` in `frontend/src/components/ProjectAssetBrowser.tsx` (sibling to `import-bsn-btn`). |
| **Keyboard shortcut** | None (button is focusable). |
| **Activation key** | `Enter` or `Space`. |
| **Expected ARIA** | `role="button"` (implicit); `aria-label="Import asset"`. |
| **Expected consequence** | Opens a hidden file picker filtered to `.aseprite,.ase,.ldtk,.tmx,.json,.png`. The selected file's name is logged and the picker resets; downstream the file is passed to the asset import pipeline (future-work: full `<ImportDialog />` wiring). |
| **Test reference** | `a11y-critical-paths.spec.ts:cp5_import_asset_button_has_aria_label_and_is_keyboard_focusable` |

**Why promoted in `cp5-import-trigger`**: the v0.110.0 cycle
(`g7-a11y-critical-paths`) declared CP-5 with a `🔴 DEFERRED` status
because no keyboard-accessible import trigger existed. The `cp5-import-trigger`
cycle (v0.110.4) added a single sibling button next to the existing
`import-bsn-btn` that opens a file picker filtered to the formats CP-5
names. The full dialog wiring (`<ImportDialog />` accepting
`onShowChangeWorkbench` callbacks) remains a separate cycle; the
**trigger** is the keyboard-accessible boundary that CP-5 requires.

**Carry-forward**: CP-5 §"What this cycle proves" stops at the trigger;
the dialog wiring (`<ImportDialog />` → Aseprite/LDtk/Tiled importers) is
the next bucket. See `docs/sddk/cp5-import-trigger/verify-report.md` and
the carry-forward note in §5.

---

## 3. Critical-path test matrix

| CP | Path doc | Test file | Test name | Status |
|----|----------|-----------|-----------|--------|
| CP-1 | §2.1 | `a11y-critical-paths.spec.ts` | `cp1_welcome_overlay_can_be_dismissed_by_keyboard` | ✅ proven (v0.110.0) |
| CP-2 | §2.2 | `a11y-critical-paths.spec.ts` | `cp2_create_entity_button_has_aria_label_and_is_keyboard_focusable` | ✅ proven (v0.110.0) |
| CP-3 | §2.3 | `a11y-critical-paths.spec.ts` | `cp3_save_action_has_keyboard_shortcut_and_aria_label` | ✅ proven (v0.110.0) |
| CP-4 | §2.4 | `a11y-critical-paths.spec.ts` | `cp4_play_mode_toggle_has_aria_label_and_is_keyboard_focusable` | ✅ proven (v0.110.0) |
| CP-5 | §2.5 | `a11y-critical-paths.spec.ts` | `cp5_import_asset_button_has_aria_label_and_is_keyboard_focusable` | ✅ proven (v0.110.4) |

This cycle (`cp5-import-trigger`) delivers **1 test** that proves CP-5
the keyboard-accessible import trigger. Combined with the earlier
`g7-a11y-critical-paths` cycle (CP-1 → CP-4), the full v1.0 a11y matrix
is **complete** with **5 tests** covering **5 declared CPs**.

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
