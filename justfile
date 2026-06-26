# Bevy 2D Editor — Development Commands

editor_crate := "crates/editor-core"
frontend     := "frontend"
wasm_out     := "frontend/src/wasm"

# Show available commands
default:
    @just --list

# Install dev tools (cargo-watch for auto-rebuild)
install-tools:
    cargo install cargo-watch wasm-pack
    cd {{frontend}} && npm install
    @echo "Dev tools installed."

# Build WASM module — dev mode (fast, no wasm-opt)
wasm:
    cd {{editor_crate}} && wasm-pack build --target web --dev --out-dir ../../{{wasm_out}}
    @echo "WASM built to {{wasm_out}}"

# Build WASM in release mode (smaller binary)
wasm-release:
    cd {{editor_crate}} && wasm-pack build --target web --release --out-dir ../../{{wasm_out}}
    @echo "WASM (release) built to {{wasm_out}}"

# Install frontend dependencies
install:
    cd {{frontend}} && npm install

# First-time setup: tools + WASM + deps
setup: install-tools wasm
    @echo "Setup complete. Run 'just dev' to start."

# Dev mode: Vite + auto-rebuild WASM on Rust save (single command!)
dev:
    cd {{frontend}} && npm run dev

# Watch mode: auto-rebuild WASM on every Rust change (run in separate terminal)
watch:
    cd {{editor_crate}} && cargo watch -s "wasm-pack build --target web --dev --out-dir ../../{{wasm_out}}"

# Production build
build: wasm-release
    cd {{frontend}} && npm run build

# Clean all build artifacts
clean:
    rm -rf target {{frontend}}/src/wasm {{frontend}}/dist
    @echo "Cleaned."

# Check Rust compilation without producing output
check:
    cargo check --target wasm32-unknown-unknown

# Install Playwright browser (first time only)
test-install:
    cd {{frontend}} && npx playwright install --with-deps chromium

# Run E2E tests (builds WASM first, then runs Playwright)
test: wasm
    cd {{frontend}} && npx playwright test

# Run E2E tests with visible browser
test-headed: wasm
    cd {{frontend}} && npx playwright test --headed

# Run a specific test file
test-one file: wasm
    cd {{frontend}} && npx playwright test {{file}}
