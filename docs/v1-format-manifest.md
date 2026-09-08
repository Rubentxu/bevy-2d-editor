# v1 Format Manifest

> **Status:** First declaration of v1 durable document formats.
> **Source cycle:** `g4-roundtrip-corpus` (A-min, sequence 319+).
> **Author:** corpus pass against `HEAD = 7b938ae` (v0.110.0 archive commit).
> **Scope:** enumerate all durable document types at their current
> (v1) schema version, declare the v0→v1 migration policy, and
> identify what's needed for a future v2.

This document is the **authoritative format registry** for the
editor's durable documents. Any change to a format's schema MUST
either bump the version constant and add a migration step, OR be
backward-compatible (new optional field with `#[serde(default)]`).

The runtime evidence for v0→v1 migration lives in
`crates/editor-model/tests/migration_corpus.rs` (7 tests as of
v0.110.0): each type has at least one historical-shape test.

---

## 1. Scope

### 1.1 What this document covers

- The 5 **durable document types** persisted to OPFS (and their
  on-disk JSON files).
- Their **schema version constants** in
  `crates/editor-model/src/migration.rs`.
- Their **v0→v1 migration policies** (where applicable).
- The **forward-compatibility rule** (ADR-0046 rule 2).

### 1.2 What this document does NOT cover

- In-memory engine types (Bevy components, runtime state).
- Wire formats for IPC (those are versioned per bridge, not per document).
- Build artifacts (compiled WASM, BSN bundles).

---

## 2. Declared v1 format list

| # | Type | Version constant | On-disk file | v0→v1 step | Migration test |
|---|------|------------------|--------------|------------|----------------|
| 1 | `SceneDocument` | `SCENE_DOCUMENT_VERSION = 1` | `scenes/<id>.json` | Materialize `instances` map + `extension_data` map | `corpus_v0_scene_document_migrates` |
| 2 | `SceneAssetDocument` | `SCENE_ASSET_DOCUMENT_VERSION = 1` | `scene_assets/<id>.json` | No-op (serde defaults) | `corpus_v0_scene_asset_document_migrates` |
| 3 | `WorldDocument` | `WORLD_DOCUMENT_VERSION = 1` | `worlds/<id>.json` | No-op (shipped at v1) | `corpus_v0_world_document_migrates` |
| 4 | `LogicGraphAsset` | `LOGIC_GRAPH_ASSET_VERSION = 1` | `logic_graphs/<id>.json` | No-op (shipped at v1) | `corpus_v0_logic_graph_asset_migrates` |
| 5 | `ProjectMetadata` | `PROJECT_METADATA_VERSION = 1` | `project.json` | Materialize `worlds` + `active_world` (ADR-0037) | `corpus_v0_project_metadata_migrates` |

All 5 types declare `version: u32` at the top level. The string
`"0.1"` (semver) parses to `0` for migration; current values are
literal `1`.

---

## 3. Version constants (canonical reference)

The version constants are declared in
`crates/editor-model/src/migration.rs`:

```rust
pub const SCENE_DOCUMENT_VERSION: u32 = 1;
pub const SCENE_ASSET_DOCUMENT_VERSION: u32 = 1;
pub const WORLD_DOCUMENT_VERSION: u32 = 1;
pub const LOGIC_GRAPH_ASSET_VERSION: u32 = 1;
pub const PROJECT_METADATA_VERSION: u32 = 1;
```

These constants are the source of truth. Any code that compares
`doc.version` to a literal must use these constants, not raw `1`.

---

## 4. Migration policy

### 4.1 Forward-compatibility rule (ADR-0046 rule 2)

All 5 durable types use the **unknown-fields-preserved** pattern:

```rust
#[serde(default, flatten)]
pub extension_data: BTreeMap<String, serde_json::Value>,
```

This guarantees that documents written by a **newer** editor
(adding fields) can still be loaded by an **older** editor that
doesn't know those fields. The unknown fields are preserved in
`extension_data` and round-tripped on the next save.

### 4.2 Backward-compatibility rule (v0 → v1)

When a field is added to a durable type:

1. The new field MUST be `#[serde(default)]` so old documents parse.
2. The v0→v1 migration step MUST be added to the corresponding
   `migrate::<type>()` function (even if it's a no-op).
3. The migration step MUST be **idempotent**: running it twice
   produces the same result.
4. The migration step MUST NOT lose data from `extension_data`.

### 4.3 Future-version rejection

Documents declaring `version > CURRENT` MUST be rejected with
`MigrationError::UnsupportedVersion { type_name, version, max }`.
This is proven by `corpus_future_version_rejected`.

---

## 5. v0 → v1 detailed migration steps

### 5.1 `SceneDocument` v0 → v1

The pre-instances shape (pre-v0.88) lacked the `instances` map.

```rust
if doc.instances.is_empty() {
    doc.instances = BTreeMap::new();
}
if doc.extension_data.is_empty() {
    doc.extension_data = BTreeMap::new();
}
```

After migration, `doc.version == 1`. All other fields preserved
(name, scene_id, entities).

### 5.2 `SceneAssetDocument` v0 → v1

The pre-flatten shape (pre-v0.85) lacked `extension_data`. Since
`#[serde(default, flatten)]` already materializes the map on parse,
the migration step is a no-op. It exists for forward documentation:
a future v2 with required fields will reuse this branch.

### 5.3 `WorldDocument` v0 → v1

The world document type shipped at v1 (ADR-0037 World Workspace).
No v0 fixtures exist in the repo. The migration step is a no-op
preserved for symmetry with other types.

### 5.4 `LogicGraphAsset` v0 → v1

The logic graph asset type shipped at v1. Same as `WorldDocument`:
no v0 fixtures; no-op migration step for forward documentation.

### 5.5 `ProjectMetadata` v0 → v1

The pre-ADR-0037 shape (pre-v0.95) lacked `worlds` and `active_world`.

```rust
if doc.worlds.is_empty() {
    doc.worlds = Vec::new();
}
if doc.active_world.is_none() {
    doc.active_world = None;
}
```

After migration, `doc.version == 1`. All other fields preserved
(name, scenes, schemas, active_scene, scene_assets).

---

## 6. Carry-forward

### 6.1 When a v2 is needed

A v2 migration is required when:

- A field is **renamed** (no longer preserved in `extension_data`).
- A field is **removed** (e.g. unused capability slot).
- A field changes **type** (e.g. `String` → `enum`).
- A field becomes **required** (breaks `#[serde(default)]`).

When that happens:

1. Bump the version constant (`SCENE_DOCUMENT_VERSION = 2`).
2. Implement the v1→v2 migration in `migrate::<type>()` (add `1 =>`
   arm that runs the v1→v2 step, then `0 =>` arm that runs v0→v1
   then v1→v2).
3. Add corpus tests covering the v1→v2 path.
4. Update this manifest with the new migration row.

### 6.2 Open questions

- **Cross-format migrations**: when a `SceneAssetDocument` is
  embedded in a `WorldDocument`, does migration need to coordinate?
  Currently no (each format migrates independently). If
  coordination is needed, this manifest will gain a §7.
- **BSN bundle versioning**: `bundled.bsn` has its own version
  scheme (per `crates/editor-bevy/src/bsn_export.rs`). Not in
  scope of this manifest; tracked under BS-3.

---

## 7. Cross-references

- `crates/editor-model/src/migration.rs` — version constants +
  `migrate` module.
- `crates/editor-model/tests/migration_corpus.rs` — 7 corpus tests.
- `docs/compatibility-policy.md` — extension API + capability
  SemVer discipline (orthogonal to document migration).
- `docs/adr/ADR-0037-world-workspace.md` — origin of `WorldDocument`.
- `docs/adr/ADR-0046-migration-and-semantic-versioning.md` —
  forward-compatibility rule 2.
- `docs/adr/ADR-0045-git-friendly-project-format-and-migrations.md` —
  intent to keep project.json Git-friendly.
