# Architecture Fitness Functions

Architecture rules are CI assertions, not conventions.

## Dependency gates

Required automated checks:

| Rule | Gate |
|---|---|
| `editor-model` has no Bevy dependency | fail CI |
| `editor-model` has no WASM/browser dependency | fail CI |
| `editor-application` has no `web-sys/js-sys` | fail CI |
| concrete OPFS types do not leak into application signatures | fail CI |
| `agent-runtime` cannot import storage/Bevy adapters directly | fail CI |
| frontend feature code cannot access raw WASM module/global `window` API | fail CI after migration grace period |
| importer crates (`editor-core::importer::*`) cannot import `EditorSession` directly (v0.93) | fail CI |
| importers must use only the typed port traits (`ImporterRegistryPort`, `ProjectStore`) not mutable session access | fail CI |

Implement with a combination of Cargo workspace dependency structure, `cargo metadata`, grep/static checks and targeted architectural tests.

## Global-state gates

After ADR-0031 migration:

```text
thread_local! allowed only in editor-wasm composition/runtime adapter modules
new domain/application statics with interior mutability forbidden
```

## Size/hotspot gates

Initial thresholds are warnings, then harden after decomposition:

- Rust production file > 30 KiB: warning + architecture review.
- Rust production file > 50 KiB: CI failure unless allowlisted (ARCH-050).
- React component > 600 LOC: warning.
- `App.tsx` target: composition only, < 300 LOC.
- backend bridge module target: no monolithic export table.

Implemented as `SIZE-1` assertion in `tools/archcheck/check.ts`.

Allowlisted hotspots (ARCH-050 tracking):
- `crates/editor-bevy/src/lib.rs` — primary barrel/API surface
- `crates/editor-bevy/src/logic_evaluator.rs` — single coherent capability
- `crates/editor-bevy/src/preview_runtime.rs` — update/schedule systems
- `crates/editor-bevy/src/processor.rs` — command processor

These limits are heuristics, not quality metrics by themselves; exceptions require rationale.

## Typed boundary gates

- forbid new `(window as any)` bridge exports;
- forbid new untyped JSON command payloads where typed DTOs exist;
- generated TypeScript bindings must match Rust protocol version;
- protocol compatibility tests run on each PR.

## Quality gates

PR:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets --locked
wasm-pack build --target web
npm run lint
npm run format:check
npm run build:check
Playwright smoke suite
architecture fitness suite
migration/golden format suite
```

Nightly/full:

```text
full Playwright
browser matrix
large-project performance corpus
BSN round-trip corpus
format migration corpus
cargo audit / cargo deny
bundle/WASM trend report
```

## Performance budgets

Record trends for:

- cold editor startup;
- project load;
- 1k/10k entity hierarchy interaction;
- viewport frame time;
- command apply latency;
- undo/redo latency;
- world search latency;
- agent context retrieval latency;
- bundle/WASM size.
