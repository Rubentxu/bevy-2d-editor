# Implementation Checklist — Every Hardening PR

## Before code

- [ ] Link roadmap hito and debt IDs.
- [ ] State observable problem.
- [ ] Add/identify characterization test.
- [ ] State format/API compatibility assumptions.
- [ ] Decide whether the change is structural or semantic; avoid mixing both.

## During code

- [ ] Keep old route only when required for transition.
- [ ] Add no new ambient globals.
- [ ] Add no new production raw bridge caller.
- [ ] Update fitness rule when a new boundary becomes enforceable.
- [ ] Avoid unrelated cleanup in migration PR.

## Verification

- [ ] Rust tests.
- [ ] wasm32 check when relevant.
- [ ] frontend static checks.
- [ ] architecture checks.
- [ ] relevant Playwright cohort.
- [ ] UAT IDs executed.
- [ ] benchmark if hot path affected.
- [ ] serialization/BSN corpus if domain/format affected.

## After code

- [ ] Update evidence template.
- [ ] Remove or reduce allowlist entry.
- [ ] Record checkpoint decision.
- [ ] Remove old route in separate cleanup PR when parity proven.
- [ ] Update roadmap status from evidence, not intent.

