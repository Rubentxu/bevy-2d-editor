# G8 — Extension compat policy runtime evidence (explore-report)

**Cycle**: `p-28fce7028ac3c497/g8-extension-compat-policy-runtime`
**Sequence**: 289 (2026-09-08)
**Path**: A-min
**Phase**: explore

## 1. Origin

The v1.0-stabilization evidence map marks **G8** (extension and
agent capability APIs have documented compatibility policy) as 🔴
red. However, on closer inspection:

- **`docs/compatibility-policy.md`** (323 lines) was committed
  2026-09-07 (`8590831` — *one day before this cycle*).
  The policy documents document format versions, extension API
  SemVer discipline, `#[non_exhaustive]` discipline, capability
  tool surface, stability levels (Stable/Beta/Experimental),
  deprecation policy (1 minor cycle), breaking change process, and
  version pinning strategy.

- The baseline evidence map (committed 2026-09-06) predates the
  policy doc, so G8 still shows 🔴 in the map.

The gap is **runtime evidence**: no test currently pins down the
runtime contract that `ExtensionManifest` is git-friendly
(JSON round-trip), has a SemVer version field, declares
capabilities, and survives deserialization losslessly. This is the
**runtime half** of G8 — the documentation half is already
complete.

This cycle closes the runtime evidence by adding a Rust integration
test that exercises `ExtensionManifest` round-trip + version
semantics.

## 2. Hypothesis

A Rust integration test in
`crates/editor-model/tests/extension_compat.rs` that asserts:

1. **JSON round-trip is byte-identical** for an `ExtensionManifest`
   with all fields populated (id, version, capabilities,
   permissions) — proves ADR-0003 forward-compat at runtime.
2. **`SemVer` parsing accepts valid versions** (`"1.0.0"`,
   `"0.1.0"`, `"2.10.5"`) and **rejects malformed ones** (empty
   string, missing parts, non-numeric segments) — proves version
   pinning rule is enforceable.
3. **Capability categories are covered**: at least one descriptor
   per category declared in ADR-0040 (commands, menus,
   validators, importers, recipes, inspectors, asset processors,
   panels, runtime diagnostics).

This pins down the runtime contract without re-documenting what
`docs/compatibility-policy.md` already declares.

## 3. Investigation

### Existing code

`crates/editor-model/src/extension.rs:240`:
```rust
pub struct ExtensionManifest {
    pub id: ExtensionId,
    pub version: SemVer,
    pub capabilities: Vec<CapabilityDescriptor>,
    pub permissions: Vec<Permission>,
}
```

`SemVer` is presumably `crates/editor-model/src/ids.rs` or similar.

### Existing tests

`crates/editor-model/tests/` has `migration_corpus.rs` (95 lines)
and other tests. No `extension_compat.rs` yet.

### What's pinned vs what's not

Pinned by existing code:
- `ExtensionManifest` shape (compile-time enforced).
- `SemVer` parse — need to verify this rejects malformed input.

Not pinned:
- JSON round-trip semantics (could regress silently).
- Capability category coverage (could be removed silently).
- Versioning semantics (could accept garbage).

## 4. Decision

- **New file**: `crates/editor-model/tests/extension_compat.rs`.
- **Three tests**:
  1. `extension_manifest_json_round_trip_is_byte_identical` —
     build a manifest with all fields, serialize to JSON, parse
     back, re-serialize, assert byte-identical.
  2. `semver_parses_valid_and_rejects_malformed` — try a battery
     of inputs and assert correct accept/reject.
  3. `capability_enum_has_builtin_categories` — assert
     `Capability::builtin_count() >= 8` and the enum variants
     include at least: Commands, Validators, Recipes, Importers,
     Inspectors, AssetProcessors, Panels, DiagnosticProviders
     (the runtime categories declared in
     `crates/editor-model/src/extension.rs:109-130`).

  ADR-0040 also lists "menus/palette entries" which is not yet
  a `Capability` variant — that's a known extension surface gap,
  not a regression. The test pins the 8 current categories without
  forbidding future additions (`#[non_exhaustive]`).

## 5. Validation status

Pending implementation. Expected:
- `cargo test -p editor-model --test extension_compat` → 3/3 pass.
- `cargo test -p editor-bevy --lib` → 414/0/1 unchanged.
- `bun run tools/archcheck/check.ts` → green.

## 6. Open questions

None. The pattern is well-established by `migration_corpus.rs`.
