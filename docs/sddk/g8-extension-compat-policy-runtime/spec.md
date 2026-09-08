# G8 — Extension compat policy runtime evidence (spec)

**Cycle**: `p-28fce7028ac3c497/g8-extension-compat-policy-runtime`
**Sequence**: 290 (2026-09-08)
**Path**: A-min
**Phase**: specify

## Acceptance criteria

### AC-G8.1 — `ExtensionManifest` JSON round-trip is byte-identical

```
$ cargo test -p editor-model --test extension_compat extension_manifest_json_round_trip_is_byte_identical

test extension_manifest_json_round_trip_is_byte_identical ... ok
```

**Given**: A constructed `ExtensionManifest` with id, version
(SemVer), capabilities (Vec<CapabilityDescriptor>), and permissions
(Vec<Permission>).
**When**: Serialized to JSON via `serde_json::to_string_pretty`,
parsed back via `serde_json::from_str`, and re-serialized.
**Then**: The first and second serializations produce identical
text. This pins down the JSON-determinism property required by
ADR-0003 (forward-compat via serde_json::Value) and ADR-0045
(Git-friendly project format).

### AC-G8.2 — `SemVer::parse` accepts valid / rejects malformed

```
$ cargo test -p editor-model --test extension_compat semver_parses_valid_and_rejects_malformed

test semver_parses_valid_and_rejects_malformed ... ok
```

**Given**: A battery of SemVer input strings.
**When**: Each is passed to `SemVer::parse(&str)`.
**Then**: Valid inputs (`"0.1.0"`, `"1.0.0"`, `"2.10.5"`) parse
successfully and round-trip via `Display`. Malformed inputs
(`""`, `"1"`, `"1.0"`, `"a.b.c"`, `"1.0.0.0"`) return `None`. This
pins down the version-pinning semantics required by the compat
policy's SemVer discipline section.

### AC-G8.3 — `Capability` enum covers the declared runtime categories

```
$ cargo test -p editor-model --test extension_compat capability_enum_has_builtin_categories

test capability_enum_has_builtin_categories ... ok
```

**Given**: The `Capability` enum in
`crates/editor-model/src/extension.rs:109-130`.
**When**: We construct at least one `CapabilityDescriptor` per
runtime category (Commands, Validators, Recipes, Importers,
Inspectors, AssetProcessors, Panels, DiagnosticProviders) and
serialize + deserialize a manifest containing all of them.
**Then**: Each category round-trips losslessly and
`Capability::builtin_count() == 8`. This pins down the
capability-tool-surface coverage from the compat policy.
