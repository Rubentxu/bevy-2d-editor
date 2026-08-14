# Specification — CI, Quality and Architecture Gates

## Workflows

### `ci-rust.yml`
- fmt;
- clippy;
- workspace tests;
- dependency/architecture checks.

### `ci-wasm-frontend.yml`
- wasm-pack build;
- TypeScript compile;
- lint/format;
- Vite production build;
- bundle budget.

### `ci-e2e-smoke.yml`
Critical Playwright paths on PR.

### `nightly-full.yml`
Full E2E, browser matrix, performance, migration corpus, BSN round-trips.

### `release.yml`
Tag-triggered production artifacts/checks.

## Required branch checks

`main` cannot merge without required PR workflows succeeding.

## Architectural checks

Implement script(s) under `scripts/architecture/` that fail if:

- forbidden Cargo dependencies appear;
- new application/model `thread_local!` appears;
- `window as any` grows after baseline;
- protocol generation/check is stale.

## Test pyramid

1. pure domain/application unit tests;
2. adapter contract tests;
3. WASM integration tests;
4. focused React tests with fake backend;
5. E2E smoke/full;
6. golden/migration/performance corpus.
