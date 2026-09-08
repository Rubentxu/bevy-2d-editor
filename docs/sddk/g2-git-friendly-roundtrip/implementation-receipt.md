# G2 — Git-friendly round-trip test (implementation-receipt)

**Cycle**: `p-28fce7028ac3c497/g2-git-friendly-roundtrip`
**Sequence**: 280 (2026-09-08)
**Path**: A-min
**Phase**: build

## Files created

```
frontend/tests/git-friendly-roundtrip.spec.ts  NEW (182 lines, 2 tests)
```

## Implementation

`frontend/tests/git-friendly-roundtrip.spec.ts`:

### Test 1: `project_json_round_trip_is_byte_identical`

Steps:
1. Load clean editor.
2. Mount `examples/platformer-minimal/` into OPFS (9 files via
   `opfs_save_file`).
3. Call `load_project()` to hydrate.
4. Call `opfs_load_file("project.json")` — get first read (raw text).
5. Normalise: `JSON.parse(raw)` → re-`JSON.stringify` with sorted
   keys + indent=2.
6. Call `opfs_save_file("project.json", normalised)`.
7. Call `opfs_load_file("project.json")` — get second read.
8. Assert second read equals normalised.

This proves the editor's persistence layer produces deterministic
text (no timestamps, no key reorderings, no whitespace churn).

### Test 2: `every_opfs_file_is_parseable_json_with_version_field`

Steps:
1. Same setup as Test 1 (mount sample).
2. Iterate over all 9 OPFS file paths.
3. For each: `opfs_load_file`, `JSON.parse`, assert top-level
   `version` field exists and is truthy.
4. Assert zero failures.

This proves ADR-0045's "every persisted document has an explicit
schema/format version" clause is honored.

Reuses:
- `waitForEditorReady` helper.
- `OPFS_FILES` table (mirrors `e2e-game-creation.spec.ts`).
- `mountSample` helper (same shape as e2e-game-creation).

## Verification

```
$ cd frontend && npx tsc --noEmit -p .
(no output: TypeScript clean)

$ cd frontend && npx playwright test --list git-friendly-roundtrip
[full] › git-friendly-roundtrip.spec.ts:89:3 › G2 — Git-friendly round-trip › project_json_round_trip_is_byte_identical
[full] › git-friendly-roundtrip.spec.ts:141:3 › G2 — Git-friendly round-trip › every_opfs_file_is_parseable_json_with_version_field
Total: 2 tests in 1 file
```

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored
```

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Acceptance

✓ `npx tsc --noEmit -p .` clean
✓ `npx playwright test --list` registers both tests
✓ `cargo test -p editor-bevy --lib` unchanged at 414/0/1
✓ `bun run tools/archcheck/check.ts` green

## G2 status

The runtime-evidence half of G2 is now closed. ADR-0045's intent
(declarative) + this test (runtime) + the existing branch
protection rule (governance) = G2 should now be ready to upgrade
from 🟡 to ✅.

The documentation half of G2 ("documented Git workflow in
CONTRIBUTING.md") is partially satisfied by the existing
trunk-based-development section. A future doc-only cycle may add
a "Git-friendly project format" subsection summarising ADR-0045's
runtime guarantees for end-users, but that's optional polish.
