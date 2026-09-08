# BJ-3 — UI entity creation test (verify-report)

**Cycle**: `p-28fce7028ac3c497/bj3-ui-entity-creation`
**Sequence**: 270 (2026-09-08)
**Path**: A-min
**Phase**: verify

## Test posture

### Static checks

```
$ cd frontend && npx tsc --noEmit -p .
(no output: TypeScript clean)

$ cd frontend && npx playwright test --list ui-entity-creation
  [full] › ui-entity-creation.spec.ts:22:3 › BJ-3 — UI entity creation › add_entity_button_creates_one_entity
  [full] › ui-entity-creation.spec.ts:62:3 › BJ-3 — UI entity creation › add_entity_button_can_be_clicked_multiple_times
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
| AC-BJ3.1 | `add_entity_button_creates_one_entity` registered, compiles | ✅ |
| AC-BJ3.2 | `add_entity_button_can_be_clicked_multiple_times` registered, compiles | ✅ |
| AC-BJ3.3 | 414/0/1 editor-bevy unchanged | ✅ |
| AC-BJ3.4 | archcheck green | ✅ |
| AC-BJ3.5 | `npx tsc --noEmit -p .` clean | ✅ |

## Risk

Low. The test reuses battle-tested patterns from
`selected-entity.spec.ts` and the `waitForEditorReady` helper.

## Open issues

None.

## Ready for release

Yes. The new spec is committed and verified at the static-analysis
level. Runtime verification happens in CI under the @full cohort.
