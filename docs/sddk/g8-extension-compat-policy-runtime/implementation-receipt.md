# G8 — Extension compat policy runtime evidence (implementation-receipt)

**Cycle**: `p-28fce7028ac3c497/g8-extension-compat-policy-runtime`
**Sequence**: 292 (2026-09-08)
**Path**: A-min
**Phase**: build

## Files created

```
crates/editor-model/tests/extension_compat.rs  NEW (180 lines, 3 tests)
```

## Implementation

`crates/editor-model/tests/extension_compat.rs`:

### Test 1: `extension_manifest_json_round_trip_is_byte_identical`

Builds an `ExtensionManifest` with id, SemVer, 8 capability
descriptors, and 2 permissions. Serializes via
`serde_json::to_string_pretty`, parses back via
`serde_json::from_str`, re-serializes. Asserts:
- The two serializations are byte-identical (no field reordering,
  no whitespace churn).
- The parsed manifest equals the original (`PartialEq`).

### Test 2: `semver_parses_valid_and_rejects_malformed`

Two assertions:
- Valid inputs (`"0.1.0"`, `"1.0.0"`, `"2.10.5"`, `"10.20.30"`)
  parse successfully and round-trip via `Display`.
- Malformed inputs (10 cases: empty, fewer/more parts, non-numeric,
  empty segments) return `None`.

### Test 3: `capability_enum_has_builtin_categories`

Two assertions:
- `Capability::builtin_count() == 8` (matches the policy's
  declared category count).
- A manifest with one `CapabilityDescriptor` per declared runtime
  category round-trips each variant's discriminant losslessly.

## Verification

```
$ cd crates/editor-model && cargo test --test extension_compat

running 3 tests
test semver_parses_valid_and_rejects_malformed ... ok
test capability_enum_has_builtin_categories ... ok
test extension_manifest_json_round_trip_is_byte_identical ... ok

test result: ok. 3 passed; 0 failed
```

```
$ cargo test -p editor-model --lib
test result: ok. 313 passed; 0 failed; 0 ignored
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

✓ `cargo test -p editor-model --test extension_compat` → 3/3 pass
✓ `cargo test -p editor-model --lib` → 313/0/0 unchanged
✓ `cargo test -p editor-bevy --lib` → 414/0/1 unchanged
✓ `bun run tools/archcheck/check.ts` green
