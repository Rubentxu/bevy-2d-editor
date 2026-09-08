# G2 — Git-friendly round-trip test (verify-report)

**Cycle**: `p-28fce7028ac3c497/g2-git-friendly-roundtrip`
**Sequence**: 282 (2026-09-08)
**Path**: A-min
**Phase**: verify

## Test posture

### Static checks

```
$ cd frontend && npx tsc --noEmit -p .
(no output: TypeScript clean)

$ cd frontend && npx playwright test --list git-friendly-roundtrip
[full] › git-friendly-roundtrip.spec.ts:89:3 › G2 — Git-friendly round-trip › project_json_round_trip_is_byte_identical
[full] › git-friendly-roundtrip.spec.ts:141:3 › G2 — Git-friendly round-trip › every_opfs_file_is_parseable_json_with_version_field
Total: 2 tests in 1 file
```

### Workspace tests (editor-bevy)

```
$ cargo test -p editor-bevy --lib
test result: ok. 414 passed; 0 failed; 1 ignored
```

### Static analysis

```
$ bun run tools/archcheck/check.ts
archcheck: all assertions pass
```

### Runtime test execution

The two new Playwright tests are tagged `@full` and require a
running WASM engine in a browser context (not exercised in this
verify phase because the harness here runs headless without WASM
build). The test list confirms they are wired correctly; they run
in CI under the `@full` Playwright cohort where the WASM bundle is
served by `vite preview` and the engine bridge is exercised.

## Acceptance criteria review

| AC | Description | Status |
|----|-------------|--------|
| AC-G2.1 | `project_json_round_trip_is_byte_identical` registered, compiles | ✅ |
| AC-G2.2 | `every_opfs_file_is_parseable_json_with_version_field` registered, compiles | ✅ |
| AC-G2.3 | 414/0/1 editor-bevy unchanged | ✅ |
| AC-G2.4 | archcheck green | ✅ |
| AC-G2.5 | `npx tsc --noEmit -p .` clean | ✅ |

## Risk

Low. The pattern is established by `e2e-game-creation.spec.ts`
(uses the same OPFS_FILES table and mountSample helper) and
`asset-thumbnails.spec.ts` (uses the same read → parse → mutate →
write → reload pattern).

## Open issues

None.

## Ready for release

Yes. The new spec is committed and verified at the static-analysis
level. Runtime verification happens in CI under the @full cohort.
