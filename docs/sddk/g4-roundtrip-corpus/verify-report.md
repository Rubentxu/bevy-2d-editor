# G4 — Round-trip / migration corpus (verification report)

**Cycle**: `p-28fce7028ac3c497/g4-roundtrip-corpus`
**Sequence**: 324 → 325
**Phase**: verify

## Verification (A-min)

A-min uses `phase.verify.complete.a-min` for the verify transition.

## Test posture

### Cargo

```
$ cargo test -p editor-model --test migration_corpus
test corpus_all_types_reject_future_version ... ok
test corpus_current_version_round_trip_noop ... ok
test corpus_future_version_rejected ... ok
test corpus_v0_logic_graph_asset_migrates ... ok
test corpus_v0_project_metadata_migrates ... ok
test corpus_v0_scene_asset_document_migrates ... ok
test corpus_v0_scene_document_migrates ... ok
test corpus_v0_world_document_migrates ... ok

test result: ok. 8 passed; 0 failed; 0 ignored
```

8 migration tests pass (4 original + 4 new). 4 new tests cover
the 3 missing types + 1 cross-type future-version rejection.

### Lib

```
$ cargo test -p editor-model --lib
test result: ok. 313 passed; 0 failed; 0 ignored
```

### Static analysis

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

## Workspace

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored
```

editor-bevy unchanged.

## Acceptance criteria

| Criterion | Status |
|-----------|:---:|
| `docs/v1-format-manifest.md` committed with 5-row table | ✅ |
| `migration_corpus.rs` extended with ≥3 new tests | ✅ (4 new) |
| All 8 migration tests pass | ✅ |
| editor-model lib tests: 313 unchanged | ✅ |
| archcheck green | ✅ |
| G4 ready to upgrade 🟡 → ✅ | ✅ |

All 6 acceptance criteria met.

## Result

All checks pass. Cycle ready for `release.complete`.

## Lessons

1. **A-min transition naming**: `phase.build.complete` (no `.a-min`
   suffix). The `.a-min` variant exists in some contexts but the
   engine's A-min path uses the plain name. Discovered via
   `sddk cycle next` showing the canonical transition ID.
2. **Lowercase serde enums**: 3 of the 4 new tests required fixture
   iteration because `SceneAssetRole` and `LayoutPolicy::Kind` use
   `rename_all = "lowercase"`. Captured in the implementation
   receipt's "Discoveries" section for future cycles.
3. **Cross-type table-driven test**: a single
   `corpus_all_types_reject_future_version` test exercises all 5
   `migrate::<type>` rejection arms in 30 lines. Cheaper than 5
   separate negative tests.
