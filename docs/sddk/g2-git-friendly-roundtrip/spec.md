# G2 — Git-friendly round-trip test (spec)

**Cycle**: `p-28fce7028ac3c497/g2-git-friendly-roundtrip`
**Sequence**: 278 (2026-09-08)
**Path**: A-min
**Phase**: specify

## Acceptance criteria

### AC-G2.1 — `project.json` round-trip is byte-identical

```
$ cd frontend && npx playwright test git-friendly-roundtrip --grep "byte_identical" --project=chromium

test project_json_round_trip_is_byte_identical ... ok
```

**Given**: A clean editor with `examples/platformer-minimal/`
mounted into OPFS and `load_project()` called once to hydrate.
**When**: The test reads `opfs_load_file("project.json")`,
normalises its JSON form (parses → re-stringifies with sorted keys
+ stable indent), writes it back via `opfs_save_file`, and reads it
again via `opfs_load_file`.
**Then**: The two reads produce byte-identical text.

This proves the project format is text-deterministic: no hidden
timestamps, no key reorderings, no whitespace churn.

### AC-G2.2 — every OPFS file is parseable JSON with `version`

```
$ cd frontend && npx playwright test git-friendly-roundtrip --grep "version_field" --project=chromium

test every_opfs_file_is_parseable_json_with_version_field ... ok
```

**Given**: Same setup as AC-G2.1 (sample mounted, `load_project`
called).
**When**: The test enumerates the four known path prefixes
(`scenes/`, `schemas/`, `assets/`, `logic_graphs/`) and reads each
file via `opfs_load_file`.
**Then**: Every file (a) parses as JSON without error and
(b) has a top-level `version` field (string or number, truthy).

This proves ADR-0045's "every persisted document has an explicit
schema/format version" is honored at runtime.
