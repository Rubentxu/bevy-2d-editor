# ADR-0012: Editor Choice — CodeMirror 6 over Monaco

## Status

Accepted (2026-07-02)

## Context

Hito 4 Order 1 introduces an in-editor code editing surface for Rust source files.
Two editor libraries were evaluated as the foundation:

- **CodeMirror 6** (`@uiw/react-codemirror` + `@codemirror/lang-rust`)
- **Monaco Editor** (VS Code's editor, `@monaco-editor/react`)

This decision is hard to reverse because it propagates into Orders 2–6 (scene↔source
navigation, build/run integration, hot reload, code-aware AI). Changing editors after
these orders would require significant migration work.

### Bundle Size Constraints

The ROADMAP specifies a risk-budget of **<200KB gzip** for the code editor addition.
This constraint rules out Monaco at the architectural level.

| Editor | Bundle (gzip) | Notes |
|--------|---------------|-------|
| Monaco (full) | ~500KB–2MB | VS Code lineage; worker-based architecture |
| CodeMirror 6 | ~130KB | Tree-shakable; modular language support |
| CodeMirror 6 + Rust | ~150KB | `codemirror-lang-rust` adds ~20KB |

**Monaco's worker-based architecture** requires separate build artifacts for the editor
web worker, which adds integration complexity in a Vite/WASM environment where
`editor-core` is compiled to WASM. CodeMirror 6 is synchronous and integrates without
worker overhead.

### Vite / WASM Compatibility

The `editor-core` crate is compiled to WASM via `wasm-pack` and bundled by Vite.
Monaco's worker-based architecture conflicts with this setup:

1. Monaco workers require separate HTTP endpoints or inline blob URLs
2. WASM bundling with Vite's worker handling adds complexity
3. CodeMirror 6's imperative API works directly in the browser without workers

CodeMirror 6 has a proven Vite/WASM integration pattern in the Rust/WASM ecosystem
(`wasm-bindgen` + `web-sys` environments).

### Extension API / Orders 2–6

Orders 2–6 extend the code editor with:
- Scene↔source navigation (jump to definition)
- Build/run integration (invoke `cargo` from editor)
- Hot reload (watch file changes)
- Code-aware AI (Ollama/OpenAI integration)

CodeMirror 6's **extension API** is imperative and composable — extensions are plain
objects that hook into the editor lifecycle. This is easier to integrate with a
WASM-backed service layer than Monaco's VS Code-compatible extension model, which
expects a full Language Server Protocol stack.

Monaco's extension API is powerful but designed for full IDE scenarios (VS Code).
For v1, CodeMirror 6's composable extension model is sufficient and far simpler.

## Decision

We choose **CodeMirror 6** via `@uiw/react-codemirror` for the following reasons:

1. **Bundle size**: ~130KB gzip fits comfortably under the 200KB risk budget
2. **Rust syntax**: `@codemirror/lang-rust` provides first-class Rust support
3. **Vite/WASM compat**: No worker overhead; imperative API integrates cleanly
   with the existing `wasm-bindgen` + `editor-core` architecture
4. **Extension model**: Composable imperative extensions are sufficient for Orders 2–6
5. **React wrapper**: `@uiw/react-codemirror` reduces the CM6 API surface for v1,
   hiding CM6's imperative extension API behind a React-friendly component API

### Theme

The theme choice (`@uiw/codemirror-theme-*` vs custom tokens matching `.editor-panel` CSS)
is deferred to visual QA spike. The selected theme must match the existing editor
panel colors and chrome.

## Considered Options

### Option A — Monaco Editor (`@monaco-editor/react`)

Rejected. Bundle size (~500KB–2MB gzip) exceeds the 200KB risk budget. Worker-based
architecture adds Vite/WASM integration complexity that is disproportionate to the
v1 scope (CRUD only, no Language Server features yet).

### Option B — CodeMirror 6 (`@uiw/react-codemirror`) — Chosen

Chosen. ~130KB gzip, no workers, imperative extension API sufficient for v1–v6.

### Option C — CodeMirror 5

Rejected. CM5 is in maintenance mode; CM6 has better tree-shaking, better Rust
support (`@codemirror/lang-rust`), and a more modern architecture.

### Option D — Ace Editor

Rejected. Larger bundle than CM6, less mature Rust mode, fewer modern features.

## Consequences

### Positive

- **Fits budget**: ~130KB gzip is well under the 200KB risk ceiling
- **Clean Vite/WASM integration**: No worker complexity; works directly with WASM
  service layer
- **Rust-first**: `@codemirror/lang-rust` is maintained by the CodeMirror team
- **Future-proof for Orders 2–6**: Composable extension API scales to navigation,
  build/run, and AI features without migration
- **Proven in Rust/WASM**: CodeMirror 6 has established patterns in `wasm-bindgen`
  environments

### Negative

- **Monaco's VS Code compatibility**: Monaco's IntelliSense/debug integration is
  more mature. CodeMirror 6 requires custom work for deep IDE features (deferred
  to Orders 2–6)
- **Smaller ecosystem**: Monaco has more third-party language modes; CodeMirror 6's
  ecosystem is growing but smaller
- **React wrapper dependency**: `@uiw/react-codemirror` adds a wrapper layer; if the
  wrapper stalls, migration to raw CM6 is straightforward but requires work

## Measured Impact (to be filled after PR 4)

| Metric | Value |
|--------|-------|
| Bundle delta (gzip) | TBD after PR 4 |
| CM6 load time | TBD |
| Rust highlighting perf | TBD |

## References

- [ADR-0011](./0011-logic-bricks-compiled-rust-controllers.md) — Logic Bricks precedent for choosing curated over generic
- `crates/editor-core/src/lib.rs` — existing `opfs_*` WASM externs pattern
- `crates/editor-core/src/source_files.rs` — source file CRUD module
- `frontend/src/components/CodeEditor.tsx` — CM6 integration point (PR 3)
- Vite worker handling: <https://vitejs.dev/guide/features.html#web-workers>
- CodeMirror 6 Rust support: <https://github.com/codemirror/lang-rust>
