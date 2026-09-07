# Test Strategy — v1 Hardening

## Test pyramid adapted to editor architecture

### 1. Pure model tests

Fast, no Bevy/browser:

- IDs/invariants;
- graph algorithms;
- document validation;
- command semantic helpers;
- serialization/migration values.

### 2. Application use-case tests

Use InMemory/Fake ports:

- EditorSession;
- transactions;
- histories;
- policies;
- cross-resource planning;
- recovery decisions.

### 3. Adapter contract suites

Run same contract against:

- InMemoryProjectStore;
- OPFS/web implementation where test environment supports it;
- future filesystem adapter.

Bevy runtime adapter receives its own projection/runtime integration tests.

### 4. WASM boundary tests

Verify:

- initialization contract;
- DTO/error mapping;
- backend capability bindings;
- no hidden target registration races.

Do not duplicate every domain test at WASM level.

### 5. Frontend component/feature tests

Use injected backend:

- successful operation;
- validation failure;
- not-ready;
- permission denied;
- persistence failure;
- slow operation/cancellation where relevant.

### 6. Playwright product cohorts

Recommended tags/configs:

```text
@smoke
@scene
@assets
@logic
@world
@runtime
@persistence
@recovery
@a11y
@full
```

## Characterization tests

Mandatory before structural migration of behavior with weak boundaries.

Characterization records externally meaningful behavior, not implementation details. After migration the same test must pass.

## Property-based testing

Strong candidates:

- graph operations;
- command inverse/apply;
- migration round trips;
- path normalization;
- override/resync laws;
- serializer determinism.

## Fault injection

Create deterministic failure points in adapters rather than trying to crash browser processes randomly for every scenario.

Examples:

```text
fail next write
fail flush
fail read of path
return stale listing
partial import resource failure
lock/availability error
```

Then retain a smaller true-browser interruption E2E cohort.

## Flaky test policy

A flaky test is a product/test-system defect, not normal background noise.

- retries disabled while diagnosing;
- classify readiness/shared state/timing/product failure;
- quarantining requires owner and exit condition;
- release gate excludes unresolved critical-path quarantine.

## Test evidence naming

Every milestone stores/report links using IDs from UAT docs, e.g.:

```text
H2.3 / UAT-ARCH-003 / PASS / commit <sha>
```

