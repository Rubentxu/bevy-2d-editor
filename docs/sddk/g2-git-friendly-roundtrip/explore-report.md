# G2 — Git-friendly round-trip test (explore-report)

**Cycle**: `p-28fce7028ac3c497/g2-git-friendly-roundtrip`
**Sequence**: 277 (2026-09-08)
**Path**: A-min
**Phase**: explore

## 1. Origin

The v1.0-stabilization evidence map marks **G2** (filesystem/Git
workflow documented and stable) as 🟡 amber:

> ADR-0045 declares intent; OPFS is single-origin today. Branch
> protection rule exists. **No filesystem-mode code; no Git
> round-trip E2E test; no documented Git workflow in CONTRIBUTING.md
> or equivalent.**

ADR-0045 commits the project to a "deterministic text representation
whenever practical" — text-based, diff-able, version-stable. The
runtime evidence is missing.

This cycle closes the **runtime-evidence** half of G2 by adding a
Playwright spec that proves:

1. The editor can write a project to OPFS as deterministic text.
2. Reading the text back, parsing it, and re-hydrating the editor
   reconstructs the same logical state.
3. The text is stable: re-saving the same logical state produces
   textually identical bytes (no timestamps, no reorderings).

(3) is the key Git-friendly property — without it, every save
would generate spurious diffs that pollute PR review.

## 2. Hypothesis

A Playwright spec that:

1. Loads a clean editor.
2. Mounts the canonical sample (`examples/platformer-minimal/`)
   into OPFS — same as `e2e-game-creation.spec.ts`.
3. Calls `load_project()` to hydrate.
4. Calls `opfs_load_file("project.json")` and parses the returned
   text — verifies it's valid JSON with the expected top-level
   keys (`scenes`, `schemas`, `scene_assets`, `version`).
5. Calls `opfs_save_file("project.json", ...)` to write the project
   back to a deterministic path (no mutations — same content).
6. Calls `opfs_load_file("project.json")` again and asserts the two
   loads are byte-identical (text-determinism).
7. Calls `load_project()` again and asserts the entity count
   matches the original (round-trip logical-state equivalence).

A separate test (or sibling assertion): read all `scenes/*.json`,
`schemas/*.json`, `assets/*.json`, `logic_graphs/*.json` and assert
that each one is parseable JSON with a top-level `version` field
(the schema/format-version declared by ADR-0045).

## 3. Investigation

### Existing patterns

`frontend/tests/e2e-game-creation.spec.ts` (272 lines) already
provides:

- `OPFS_FILES` array — list of files to mount into OPFS.
- `mountSample(page)` helper — calls `opfs_save_file` for each.
- `load_project()` invocation after mount.

`frontend/tests/asset-thumbnails.spec.ts:155-180` already
demonstrates the read → parse → mutate → write → reload pattern.

### Gaps in existing coverage

- No existing test asserts that `project.json` text is **stable
  across save/load cycles** (text-determinism).
- No existing test asserts that **every OPFS-mounted file** is
  parseable JSON with a `version` field.
- No existing test asserts that `load_project()` after a no-op
  re-save produces the same entity count as the original (logical
  round-trip).

### Naming and placement

`frontend/tests/git-friendly-roundtrip.spec.ts`. Tagged `@full` (WASM
engine required). Lives next to `e2e-game-creation.spec.ts`.

## 4. Decision

- **New file**: `frontend/tests/git-friendly-roundtrip.spec.ts`.
- **Two tests**:
  1. `project_json_round_trip_is_byte_identical` — mount sample,
     load, save same content back, read again, assert byte equality.
  2. `every_opfs_file_is_parseable_json_with_version_field` —
     iterate over `scenes/`, `schemas/`, `assets/`, `logic_graphs/`
     paths and assert each is JSON-parseable with `version` at top
     level.

These two tests close the runtime-evidence gap for G2. The
documentation half (a "Git-friendly project format" section in
`CONTRIBUTING.md`) is out of scope for this cycle — separate
doc-only cycle later.

## 5. Validation status

Pending implementation. Expected:
- New spec compiles (`tsc --noEmit` clean).
- Both tests pass under the `@full` cohort.

## 6. Open questions

None. The pattern is well-established by
`e2e-game-creation.spec.ts` and `asset-thumbnails.spec.ts`.
