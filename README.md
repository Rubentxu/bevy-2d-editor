# Bevy 2D Editor

[![CI](https://github.com/Rubentxu/bevy-2d-editor/actions/workflows/ci.yml/badge.svg)](https://github.com/Rubentxu/bevy-2d-editor/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Version](https://img.shields.io/github/v/tag/Rubentxu/bevy-2d-editor?label=version)](https://github.com/Rubentxu/bevy-2d-editor/releases)

Bevy 2D Editor is a browser-based authoring environment for building Bevy 2D scenes, reusable Scene Assets, component schemas, layers, Logic Bricks, and Rust-aware game workflows. A React and TypeScript interface drives a Rust editor core compiled to WebAssembly, while project data is stored locally in the browser with OPFS and exported to Bevy-oriented formats.

## Quickstart

```sh
git clone https://github.com/Rubentxu/bevy-2d-editor.git
cd bevy-2d-editor
just setup
just dev
```

Open <http://localhost:5173> in Chromium or Chrome.

## Features

| Milestone | Production capability |
| --- | --- |
| Hito 0 | Scene document, hierarchy, inspector, schemas, undo/redo, preview, and persistence |
| Hito 1 | AI-assisted editing, Rust/BSN export, multi-scene projects, and Scene Asset foundations |
| Hito 2 | Asset browser, Scene Asset authoring and placement, overrides, validation, layers, tiles, and auto-layers |
| Hito 3 | `.bsn` import/export workflows and level inspector UX |
| Hito 4 | CodeMirror Rust editor, source integration, asset pipeline, play mode, and data hot reload |
| Hito 5 | Code-aware AI context across Rust source and editor assets |
| Hito 6 | Integrated production workflow hardening and editor capabilities |
| Hito 7 | Scene Component authoring UX, catalog picker, validation, and placement helpers |
| Logic Bricks | Sensor → Controller → Actuator graphs backed by compiled Rust evaluators |

See [docs/ROADMAP.md](docs/ROADMAP.md) for the detailed version history and status.

## Architecture

```text
+---------------- React / TypeScript ----------------+
| panels, hooks, CodeMirror, React Flow, Playwright   |
+-------------------------+---------------------------+
                          | wasm-bindgen
+-------------------------v---------------------------+
| Rust editor-core / WASM                             |
| scene model | commands | schemas | BSN | preview    |
| scene assets | instances | layers | Logic Bricks    |
+-------------+--------------------------+-------------+
              |                          |
       browser OPFS                Bevy preview
   project.json + assets          single canvas/app
              |
       optional Rust axum AI proxy -> Ollama/OpenAI
```

## Development commands

Run `just` to list all available recipes.

| Command | Purpose |
| --- | --- |
| `just setup` | Install `cargo-watch`, `wasm-pack`, frontend dependencies, and build development WASM |
| `just dev` | Start the Vite development server |
| `just watch` | Rebuild WASM when Rust source changes |
| `just wasm` | Build development WASM |
| `just wasm-release` | Build optimized release WASM |
| `just build` | Build the production frontend |
| `just check` | Check the Rust WASM target |
| `just clean` | Remove Rust, WASM, and frontend build output |
| `just ai-proxy` | Start the optional AI proxy |

## Testing and quality

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace --all-targets --release --locked
cd frontend && npm run lint
cd frontend && npm run format:check
cd frontend && npm run build:check
just test
```

Install Chromium once with `just test-install`. Run one Playwright file with `just test-one tests/smoke.spec.ts`, or use `just test-headed` for a visible browser.

## Documentation

- [User Guide](USER_GUIDE.md)
- [Contributing](CONTRIBUTING.md)
- [Security Policy](SECURITY.md)
- [Changelog](CHANGELOG.md)
- [Architecture decisions](docs/adr/README.md)

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a branch or pull request.

## License

The Rust editor core declares `MIT OR Apache-2.0`. You may use this project under either license at your option.
