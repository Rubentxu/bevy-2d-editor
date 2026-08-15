# ADR-0048: ProjectStore v1 Is a Synchronous Port

## Status

Accepted — 2026-08-15 — v0.87 (`v0.87-architecture-foundation`)

## Context

The v0.87 Architecture Foundation plan (PR4, `editor-application` + ProjectStore) introduces an editor-side application crate that owns the project storage boundary (ADR-0031, ADR-0033). The ProjectStore port needs to serve two concrete implementations:

1. `InMemoryProjectStore` — used by unit and contract tests, must be trivially correct, no async runtime required.
2. `OpfsProjectStore` — production adapter that wraps OPFS calls through `wasm_bindgen_futures`.

The design phase considered both sync and async surface shapes for the trait. The async version is more honest about OPFS's true cost (every call is a Promise that resolves on the JS event loop) but adds three concrete costs:

- An `async-trait` crate dependency and the `Box<dyn Future + Send>` allocations it incurs.
- Test ergonomics: every `InMemoryProjectStore` test becomes `.await`-shaped, complicates property-based tests with `proptest`, and complicates the contract test runner.
- An OPFS adapter that has to bridge `Promise<T>` → sync return, which is awkward on `wasm32-unknown-unknown` (no `block_on`) and requires `js_sys::Promise::then` chaining inside an `async fn` body.

For v1, the OPFS adapter does not need to be sync at the public API: it can wrap `wasm_bindgen_futures` calls in `await` internally and return `T` synchronously via `JsFuture::from(promise).await` inside an `async fn` body that the caller never sees.

## Decision

The `ProjectStore` trait in v1 is synchronous with five methods:

```text
fn list(&self, prefix: &str) -> Result<Vec<StoreEntry>, StoreError>;
fn read(&self, path: &str) -> Result<Vec<u8>, StoreError>;
fn write(&self, path: &str, bytes: &[u8], atomic: bool) -> Result<(), StoreError>;
fn delete(&self, path: &str) -> Result<(), StoreError>;
fn exists(&self, path: &str) -> Result<bool, StoreError>;
```

- `InMemoryProjectStore` is the canonical test implementation. Backed by `RwLock<HashMap<String, (Vec<u8>, u64)>>`. All five methods plus atomic-write-rollback are covered by unit tests.
- `OpfsProjectStore` is a `#[cfg(target_arch = "wasm32")]`-gated stub in PR4 (body may be `unimplemented!("...")` until v0.88). The stub demonstrates the wiring shape; full wiring is deferred to v0.88 when call-site migration happens.
- Existing OPFS call sites in `editor-core::lib.rs` and `logic_graph.rs` are NOT migrated in v0.87 (per design decision D2 and proposal locked decision `pr4-caller-migration-deferred`). They keep their direct `js_*` extern calls until v0.88.

If v0.88 OPFS adapter proves painful under sync shape, the migration path is:

1. Add a new `AsyncProjectStore` trait alongside `ProjectStore`.
2. Port `OpfsProjectStore` to async.
3. Add `#[async_trait]` to both and keep sync as a `block_on` wrapper.

This migration is documented as a follow-up ADR candidate, not as a current decision.

## Considered options

### Async ProjectStore v1
Rejected for v1: `async-trait` dependency, awkward test ergonomics, and an OPFS adapter that needs internal bridging anyway. The benefits are honest I/O modeling, but no caller in v0.87 actually needs concurrent I/O.

### Sync ProjectStore v1
Accepted: trivial `InMemoryProjectStore`, no runtime deps, OPFS adapter can do `await` internally. Callers block on a single store call, which is acceptable because editor storage operations are user-initiated and not on a hot frame loop.

### Both sync and async traits in v1
Rejected: over-engineering for a single-adapter port. Defer until a second async consumer exists.

## Consequences

- No `async-trait` dependency in `editor-application`.
- `InMemoryProjectStore` tests are plain `#[test]` functions with no `.await`.
- The OPFS adapter implementation cost is deferred but visible (the stub is in the PR4 deliverable).
- v0.88 may add `AsyncProjectStore` without breaking v1 callers because sync `ProjectStore` remains the default port.
- Existing OPFS call sites in `editor-core` continue to use the raw `js_*` externs until v0.88; the port does not introduce a partial migration risk in v0.87.

## References

- ADR-0031 — Explicit EditorSession Replaces Domain-Level Global State
- ADR-0033 — ProjectStore Port with OPFS and Filesystem Adapters
- ADR-0030 — Compile-Time Hexagonal Crate Boundaries
- v0.87 cycle spec — `docs/roadmaps/v0.87-architecture-foundation.md`
- v0.87 design — `cycles/v0.87-architecture-foundation/design.md` (decision D2, risk R2)