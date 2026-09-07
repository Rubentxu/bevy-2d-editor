# Compatibility Policy

This document is the **v1.0 contract** for what the editor guarantees to keep
working, how compatibility is versioned, and how breaking changes are made.

It applies from the v1.0 declaration forward. Earlier releases (v0.x) followed
ADR-0005 §Implementation Direction rule 7: *"Do not add compatibility shims
unless a later product milestone introduces real external users or saved
projects that must be preserved."* v1.0 introduces those external users; the
shims and policies below take effect.

## Scope

Three contract surfaces are versioned under this policy:

1. **Document format versions** — the on-disk JSON of every persisted editor
   document. Loading an older document is a forward-compatibility concern;
   loading a newer one is intentionally rejected.
2. **Extension API** — the `ExtensionManifest` shape, `Capability` enum,
   `Permission` model, and the `register_extension_wasm` /
   `list_extensions_wasm` / `unregister_extension_wasm` /
   `submit_plugin_change_set_wasm` wire surface used by built-in extensions
   and (post-rollout) third-party extensions (ADR-0040).
3. **Capability tool surface** — the typed `EditorBackend` API groups
   consumed by the React frontend (`SceneApi`, `SceneAssetApi`, `WorldApi`,
   `LogicApi`, `RuntimeApi`, `CodeApi`, `ValidationApi`, `SearchApi`,
   `ChangeApi`, `ProjectApi`) and the `BackendError` envelope that wraps
   every failure (`docs/specs/typed-editor-backend.md`). The agent runtime
   consumes this same surface through typed capabilities
   (ADR-0043 — agent-runtime capability boundary).

The editor product version (e.g. `v1.0.0`, `v1.1.0`) and the document format
versions are **independent**. A patch-level editor release may bump a
document format version; a minor editor release may leave it unchanged.

## Document format versions

All five core document types are at **format version 1** as of v1.0.
Versions are monotonic `u32`, declared as constants in
`crates/editor-model/src/migration.rs`:

| Document | Constant | Current |
|---|---|---|
| `SceneDocument` | `SCENE_DOCUMENT_VERSION` | 1 |
| `SceneAssetDocument` | `SCENE_ASSET_DOCUMENT_VERSION` | 1 |
| `WorldDocument` | `WORLD_DOCUMENT_VERSION` | 1 |
| `LogicGraphAsset` | `LOGIC_GRAPH_ASSET_VERSION` | 1 |
| `ProjectMetadata` | `PROJECT_METADATA_VERSION` | 1 |

`ProjectMetadata` additionally carries a free-form `version: String`
field (e.g. `"0.1"`) used as a project-level *semantic* tag for the
project format. This is the project-format version visible in
`project.json` and is bumped independently of `PROJECT_METADATA_VERSION`,
which tracks only the on-disk schema of the metadata document itself.

### Loading rules

- **Forward compatibility (load older).** Opening a document at version
  `N < CURRENT_VERSION` runs `migrate::<type>(N, &mut doc)` through every
  intermediate step up to `CURRENT_VERSION`. Migrations are typed
  pure functions per ADR-0046 rule 4 and SEM-5
  (`docs/specs/semantic-editor-model.md`); they never mutate the source
  file until the new version is committed atomically (see
  `docs/v1.0-stabilization-evidence-map.md` P2 for the OPFS atomic-write
  contract).
- **Backward compatibility (load newer).** Opening a document at version
  `N > CURRENT_VERSION` returns `MigrationError::UnsupportedVersion` and
  **fails loudly**. The editor never silently coerces, drops, or rewrites
  a newer format — doing so would corrupt future data the editor does not
  yet understand.
- **Unknown fields preserved on load** per ADR-0003: when a schema
  changes and a Component Instance has fields the new schema does not
  recognise, those fields MUST be preserved and marked orphaned —
  never auto-deleted. This applies at the *field* level inside any
  document; it complements the *format* level forward-compat above.
- **Serde defaults.** New optional fields use `#[serde(default)]` so
  documents written before the field existed still parse. Existing
  examples: `ProjectMetadata.schemas`, `ProjectMetadata.active_scene`,
  `ProjectMetadata.scene_assets`, `ProjectMetadata.worlds`,
  `ProjectMetadata.active_world` (`crates/editor-model/src/project_metadata.rs`).
  This is the field-level migration mechanism and is the preferred path
  for additive changes that do not warrant a format version bump.

### When to bump a format version

Bump `*_VERSION` when any of the following is true:

- A field is **renamed** (semantic change, not just a typo — the new
  name carries a different meaning).
- A field is **removed** (semantic loss; downstream consumers depend
  on the field existing).
- The semantic type of an existing field **changes** (e.g. `string →
  enum`, `pixels → world units`).
- A previously-`#[serde(default)]` field becomes semantically **required**
  (absence now means a meaningful failure mode, not "empty").

Do **not** bump `*_VERSION` for:

- Adding a new field with `#[serde(default)]` (additive change, parses
  forward through serde defaults).
- Tightening validation on an existing field (validation tightening is
  a behavioural change, not a format change; release-note it instead).
- Adding a new optional field with `#[serde(default)]` to a struct that
  is itself already default-tolerant.

When a bump is required, write a `migrate::<type>` step in
`crates/editor-model/src/migration.rs`, add a test
(`migrate_vN_to_vN_plus_1_<type>`), and document the change in
`CHANGELOG.md` Unreleased → Migration section.

## Schema registry compatibility

The Component Schema Registry follows the same field-level forward-compat
discipline as the document layer (ADR-0003). Schemas are JSON-described
and stored in `schemas/<type_id>.schema.json` files; the registry
version is **bumped together with the schema format** and lives in the
schema file itself (`schemaVersion` field).

Adding a new optional field to a schema is non-breaking.
Renaming a field, changing its type, or removing it is a breaking change
for any project containing Component Instances of that schema and
requires the migration discipline above (the project's `SceneDocument`
becomes a v2 of that schema).

`SceneComponent` schemas (`docs/specs/typed-editor-backend.md`,
ADR-0016) bind to a `SceneAssetDocument`; the binding is part of the
schema's identity. Re-binding to a different scene asset is a breaking
change for any project holding instances of that schema.

## Extension API

The Extension API is governed by `crates/editor-model/src/extension.rs`
and ADR-0040.

### SemVer discipline

`ExtensionManifest.version` is a `SemVer` (`major.minor.patch`) with the
following semantics:

- **Major bump** — breaking change to the manifest shape, capability
  enum, permission model, or wire surface. Existing extensions MUST be
  re-compiled against the new SDK. The editor MUST bump its minimum
  supported SDK major version.
- **Minor bump** — additive change (new variant of `Capability` enum,
  new optional field on `CapabilityDescriptor`, new optional field on
  `Permission`). Existing extensions continue to load and run
  unmodified; unknown variants are skipped by `list_extensions` per the
  `#[non_exhaustive]` rule below.
- **Patch bump** — bug fix that does not change the observable
  contract; existing extensions continue to load and run unmodified.

### `#[non_exhaustive]` discipline

`Capability`, `CapabilityDescriptor`, `Permission`, and the related
enums are all marked `#[non_exhaustive]`. Future SDK versions add
variants without an SDK major bump; extensions written against older
SDK versions simply do not declare the new capability.

### Permissions

The `(PermissionArea, PermissionScope, optional resource glob)`
envelope (`docs/specs/editor-extension-sdk.md`) is the security
contract. `extension:<id>` is the canonical actor prefix for
`ChangeOrigin::Plugin` ChangeSets; `TransactionKernel::apply_atomic`
re-checks declared permissions at apply time.

Extensions do not receive arbitrary project-root filesystem access by
default; access is per-Permission, and resource globs further scope
read/write/propose access to specific paths.

### Rollout gate

Three built-in extensions are required to ship with every v1.x release
(`docs/specs/editor-extension-sdk.md`):

1. `builtin.logic-bricks.controllers` — Logic Bricks RustController.
2. `builtin.logic-recipes` — built-in recipe pack.
3. `builtin.scene-validator` — scene-document validator.

If any of these cannot be built against the new SDK, the release is
**blocked** (the SDK is broken from the editor's own perspective).

## Capability tool surface

The `EditorBackend` API groups are the contract between the React
frontend and the Rust backend. The contract is:

- **Group stability.** Once a capability group is declared Stable (see
  *Stability levels* below), removing a method is a breaking change.
  Adding a method is non-breaking (frontend code that does not call
  the new method is unaffected).
- **`BackendError` envelope stability.** Every failure returned by
  every capability method MUST flow through the `{ code, message,
  resource?, details? }` envelope (`docs/specs/typed-editor-backend.md`).
  Frontend code MUST NOT parse Rust error strings to determine failure
  type. Adding new fields to `BackendError` is non-breaking; changing
  the meaning of an existing field is breaking.
- **Testability.** Every capability group MUST be representable by a
  `FakeEditorBackend` for unit and integration tests; the production
  WASM is not required for component-level tests.

The legacy `window as any.*` test-bridge surface (used by Playwright
specs) is **not part of this contract**. Test bridges are an internal
debugging interface and may be added, renamed, or removed without
notice.

## Stability levels

Each contract surface above carries a stability tag. A surface may be
**Stable**, **Beta**, or **Experimental**.

### Stable

The full compatibility contract applies: forward-compat on load, loud
fail on backward-incompat, 12-month deprecation window, breaking-change
process below. As of v1.0:

- All five document format versions are Stable.
- The Extension API (manifest + capability + permission shape + wire
  surface) is Stable.
- The `EditorBackend` capability groups are Stable.

### Beta

Forward-compat and loud-fail rules still apply. Breaking changes may
occur in a minor editor release without a deprecation window, but MUST
be called out in `CHANGELOG.md` Unreleased → Breaking Changes. Beta
surfaces ship behind a feature flag or capability flag and MUST be
opt-in. Promote to Stable after one minor release with no breaking
changes.

### Experimental

No compatibility guarantees. May be removed in the next patch release
without notice. Experimental surfaces are documented as such in the
relevant spec (`docs/specs/*.md`) and MUST NOT be used by built-in
extensions.

## Deprecation policy

Deprecated contract elements follow a **12-month deprecation window**,
measured from the editor release that first marks them deprecated to
the editor release that removes them.

The window proceeds in three stages:

1. **Announce (month 0).** Mark the element `#[deprecated(since = "X.Y.Z",
   note = "…")]` in the relevant type/function, add a `Deprecated` entry
   under the matching version in `CHANGELOG.md`, and (if user-visible)
   add a `ValidationIssue` warning surface that lists the deprecation
   for any project currently using the element.
2. **Soft-deprecate (month 3).** Emit a runtime warning whenever the
   deprecated element is used in the editor's own code or by an
   extension (logged at WARN level). Continue accepting it.
3. **Hard-deprecate (month 9).** Emit an error in development builds
   (`#[cfg(debug_assertions)]`); continue accepting it in release
   builds. Surface as a `ValidationIssue::Error` in the editor.
4. **Remove (month 12).** Delete the element in a minor or major editor
   release, bump the relevant format version or SDK major as required,
   and write the migration guide in `CHANGELOG.md` Released section.

The 12-month clock starts when the deprecation is **announced in a
release**, not when the deprecation is committed to `main`.

## Breaking change process

A change to any Stable surface is breaking and MUST follow this
process:

1. **ADR.** Write or amend an ADR under `docs/adr/`, naming the
   contract surface affected and the rationale.
2. **Migration path.** Provide a migration that takes existing data to
   the new shape without loss. For document formats, this is a new
   `migrate::<type>` step. For extensions, this is an SDK minor or
   major bump plus a porting guide in `CHANGELOG.md`. For the
   capability tool surface, this is a wrapper method that preserves
   the existing `BackendError` contract.
3. **Changelog entry.** Add an entry under `CHANGELOG.md` Unreleased
   → Breaking Changes (or → Migration for additive migration-only
   releases).
4. **Gate test.** Add a deterministic test that locks the breaking
   change in: a Rust unit/integration test for format migrations; a
   Playwright `@full` spec for capability surface changes; a Rust
   extension-port test for extension API changes.
5. **Deprecation window.** If the change removes an element, run the
   12-month window above. Pure additions are exempt from the window.
6. **Evidence map update.** Update `docs/v1.0-stabilization-evidence-map.md`
   (or its successor) to reflect the new contract surface and bump any
   gate that closes.

## Version pinning strategy

Editor releases follow semantic versioning. The mapping to this policy:

- **Major release** (`v1.0.0` → `v2.0.0`) — may break any Stable surface,
  subject to the 12-month deprecation window.
- **Minor release** (`v1.0.0` → `v1.1.0`) — may add to any Stable surface
  and may break Beta surfaces; may not break Stable surfaces.
- **Patch release** (`v1.0.0` → `v1.0.1`) — bug fixes only; no
  observable contract change.

The compatibility policy itself is part of the Stable surface. A change
to this document is itself a breaking change and follows the process
above.

## Cross-references

- **Document format versions** — `crates/editor-model/src/migration.rs`.
- **Extension API** — `crates/editor-model/src/extension.rs`,
  `docs/specs/editor-extension-sdk.md`, ADR-0040.
- **Capability tool surface** — `docs/specs/typed-editor-backend.md`.
  The agent side is governed by ADR-0043.
- **Forward compatibility (field level)** — ADR-0003.
- **Migration framework (SEM-5)** — `docs/specs/semantic-editor-model.md`,
  ADR-0046.
- **Schema registry** — `crates/editor-model/src/schema.rs`.
- **SceneComponent binding** — ADR-0016,
  `crates/editor-model/src/schema.rs::ComponentSchema::bound_scene_asset_ref`.
- **Atomic write contract (P2)** — `docs/v1.0-stabilization-evidence-map.md`
  P2, `CHANGELOG.md` Unreleased → v1.0-stabilization P2.
- **Release evidence map** — `docs/v1.0-stabilization-evidence-map.md`.
- **Project-level semver tag** — `ProjectMetadata.version` (string),
  distinct from `PROJECT_METADATA_VERSION` (u32, format schema).
