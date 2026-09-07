# UAT — Architecture & UX Hardening

## Purpose

These UAT cases verify externally meaningful architecture outcomes. They complement unit/integration tests; they are not substitutes for them.

---

## UAT-CI-001 — Architecture checker really executes

**Precondition:** clean main candidate.

**Steps:**

1. Run Architecture Fitness workflow.
2. Confirm dependency checker step executes.
3. In checker unit fixture, introduce forbidden application→storage dependency.
4. Run checker test.

**Expected:** real repository passes; violating fixture fails with precise edge message.

---

## UAT-CI-002 — Gate bootstrap failure is visible

1. Force checker installation/setup failure in a workflow test branch.
2. Observe workflow result.

**Expected:** workflow is red and evidence clearly says checker did not execute; no report claims architecture passed.

---

## UAT-ARCH-001 — Application builds without web storage adapter

1. Build/test `editor-application` in a configuration where `editor-storage-web` is absent from its dependency graph.
2. Run application unit tests with InMemory ports.

**Expected:** application behavior compiles/runs without OPFS/browser runtime.

---

## UAT-ARCH-002 — Application builds without Bevy adapter

Same pattern as UAT-ARCH-001.

**Expected:** pure use cases do not require `editor-bevy`.

---

## UAT-ARCH-003 — No hidden initialization order for scene operations

1. Start editor normally.
2. Wait only on the explicit backend readiness contract.
3. Create/load/edit/save a scene.
4. Repeat 20 cold starts in Playwright.

**Expected:** no `project store not initialized`, race, random polling or half-ready call.

---

## UAT-ARCH-004 — Independent sessions

1. Create two `EditorSession` instances with different InMemory stores.
2. Load/create different scenes/assets.
3. Mutate both interleaved.

**Expected:** no selection, history, catalog, validation or file leakage between sessions.

---

## UAT-ARCH-005 — No production bridge bypass

1. Run frontend static architecture checker.
2. Search production feature/component code for raw WASM/global mutation calls.

**Expected:** zero non-allowlisted callers outside backend adapter layer.

---

## UAT-ARCH-006 — Typed backend drift detection

1. In a test fixture change `SceneApi.renameEntity` signature.
2. Run `tsc --noEmit`.

**Expected:** all invalid consumers fail compilation; there is no stringly runtime-only failure.

---

## UAT-ARCH-007 — Single mutation result parity

For representative Scene/Asset/Logic commands:

1. Execute legacy fixture result from frozen baseline.
2. Execute final kernel path.
3. Compare semantic document, history, dirty state and validation effects.

**Expected:** declared parity except explicitly documented bug fixes.

---

## UAT-ARCH-008 — Policy cannot be bypassed

1. Create a mutation requiring review/permission.
2. Attempt through human UI, plugin/import simulation and direct capability API.

**Expected:** all enter the same authorization/transaction rules; no alternative production dispatcher bypasses policy.

---

## UAT-UI-001 — Hierarchy keyboard navigation

Using keyboard only:

1. Focus hierarchy.
2. Move up/down.
3. Expand/collapse.
4. Rename with F2.
5. Multi-select supported range/toggle.
6. Delete with confirmation behavior.

**Expected:** focus is visible and actions match mouse functionality for critical flow.

---

## UAT-UI-002 — Tree-aware search

1. Create a nested hierarchy at least 5 levels deep.
2. Search for leaf-only name.

**Expected:** matching leaf and ancestors remain visible; hierarchy context is understandable; clearing search restores prior expansion state.

---

## UAT-UI-003 — Reparent through typed feature API

1. Drag an entity to a new parent.
2. Verify command succeeds and preview updates.
3. Undo.
4. Redo.

**Expected:** no direct bridge-specific metadata is required in Hierarchy component; behavior remains reversible.

---

## UAT-UI-004 — Multi-inspector mixed fields

1. Select entities with common component and mixed vector values.
2. Confirm mixed state per relevant subfield where implemented.
3. Overwrite one field for selection.
4. Undo.

**Expected:** user understands which values are mixed and scope of overwrite.

---

## UAT-UI-005 — Validation quick navigation

1. Produce an entity/component validation error.
2. Open Validation Center or inline issue.
3. Navigate to affected entity/field.
4. Apply quick fix if available.

**Expected:** context and focus move to actionable target; issue disappears after valid correction.

---

## UAT-WORKSPACE-001 — Dirty cross-document navigation

1. Modify a Scene Asset without saving.
2. Navigate to scene/logic/code document.
3. Exercise Save/Discard/Cancel.

**Expected:** no data loss; active document and dirty state remain truthful.

---

## UAT-WORKSPACE-002 — Runtime mode orthogonality

1. Open scene and related logic/source documents.
2. Enter play.
3. Stop play.

**Expected:** runtime state does not corrupt document selection/open-document state.

