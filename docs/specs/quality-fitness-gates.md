# Specification — Quality and Architecture Fitness Gates

## Goal

Make architectural intent and v1 quality criteria executable.

## Gate families

### QG-1 Rust baseline

```bash
cargo fmt --all -- --check
cargo test --workspace --all-targets --release --locked
cargo check -p editor-model --target wasm32-unknown-unknown --locked
cargo check -p editor-wasm --target wasm32-unknown-unknown --locked
```

`clippy -D warnings` should be introduced by a ratchet strategy if the current baseline is not clean.

### QG-2 Dependency graph

A checker based on `cargo metadata` verifies allowed crate edges. Regex checks remain supplemental, not authoritative.

### QG-3 Global state ratchet

Count/locate approved global state patterns. New occurrences outside composition/runtime allowlists fail CI. The allowlist must monotonically shrink.

### QG-4 Frontend boundary

Static check forbids backend bridge calls from visual components/feature UI outside approved adapters.

### QG-5 Frontend static

```bash
npm run format:check
npm run lint
npx tsc --noEmit
npm run build:check
```

### QG-6 E2E cohorts

- smoke;
- scene authoring;
- persistence/recovery;
- asset/logic/world;
- accessibility;
- full regression.

Smoke and architecture gates must run per PR. Heavy cohorts may be split by PR/nightly depending on cost, but release requires all mandatory cohorts on the same candidate commit.

### QG-7 Contract corpus

- project serialization;
- BSN;
- migration;
- transaction atomicity;
- importer/reimport.

### QG-8 Performance

Benchmarks use versioned budgets and store result artifacts. A threshold increase requires rationale/ADR.

## Reliability rule

A gate that fails before executing its checker is considered **not operational**, not a successful architecture control.

## Evidence bundle

Each release candidate records:

```text
commit SHA
Rust gates
frontend gates
architecture gates
Playwright cohorts
benchmark report
data recovery report
format/BSN corpus report
known accepted debt
```

