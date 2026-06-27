# AI-Assisted Editing

Natural-language scene editing powered by an LLM (OpenAI GPT-4o or Ollama) via a local Rust HTTP proxy.

---

## Setup

### 1. Set your OpenAI API key

```bash
export OPENAI_API_KEY=sk-...
```

For persistent setup, add to your shell profile (`~/.bashrc`, `~/.zshrc`):

```bash
echo 'export OPENAI_API_KEY=sk-...' >> ~/.bashrc
source ~/.bashrc
```

### 2. Build WASM (first time only)

```bash
just wasm
```

### 3. Start the AI proxy

```bash
just ai-proxy
```

The proxy starts on `http://localhost:11435` by default.

**Alternative: custom port**
```bash
just ai-proxy-port PORT=11436
```

**Alternative: use Ollama instead of OpenAI**

Set `OLLAMA_BASE_URL` (default: `http://localhost:11434`):

```bash
OLLAMA_BASE_URL=http://localhost:11434 just ai-proxy
```

### 4. Start the editor

```bash
just dev
```

Navigate to `http://localhost:5173`. Click the **✨ AI** button in the top bar to open the AI Assistant panel.

---

## Usage

### Opening the AI Panel

Click the **✨ AI** button in the top bar. The panel opens on the left side of the editor.

### Submitting a Prompt

1. Type a description of the scene change you want in the textarea (e.g., "create a player sprite at x=100")
2. Click **Submit** or press `Ctrl+Enter` / `Cmd+Enter`
3. Wait for the AI response — a loading spinner appears while the proxy processes the request

### Reviewing a Proposal

When the AI returns a proposal, a **ProposalCard** appears in the panel showing:

- **Rationale** — why the AI suggested these changes
- **Model** — which AI model was used (e.g., `gpt-4o`)
- **Commands** — the list of typed editor commands that would be applied
- **Apply** / **Discard** buttons

### Applying a Proposal

Click **Apply** to dispatch each command in the proposal to the editor's command system. Commands are applied sequentially; if any command fails, the proposal stays visible with error annotations.

### Discarding a Proposal

Click **Discard** to dismiss the proposal without applying any changes. The scene state is unchanged.

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  React Frontend (TypeScript)                         │
│  AIAssistantPanel → useAIAssistant → ai-assistant.ts │
│                                                     │
│  POST /v1/propose  ──→  fetch()                     │
└──────────────────────────────┬──────────────────────┘
                               │ HTTP (CORS)
┌──────────────────────────────▼──────────────────────┐
│  Rust Axum Proxy  (crates/ai-proxy)                │
│  POST /v1/propose  →  OpenAI / Ollama              │
│  Holds OPENAI_API_KEY server-side                   │
└─────────────────────────────────────────────────────┘
```

### Key Files

| File | Role |
|------|------|
| `crates/ai-proxy/src/lib.rs` | Proxy crate entry |
| `crates/ai-proxy/src/handlers/propose.rs` | `POST /v1/propose` handler |
| `crates/ai-proxy/src/openai/mod.rs` | OpenAI API client (function calling) |
| `crates/ai-proxy/src/context/mod.rs` | System prompt builder with scene + schema context |
| `frontend/src/services/ai-assistant.ts` | Frontend fetch client for `/v1/propose` |
| `frontend/src/components/AIAssistantPanel.tsx` | React panel component |
| `frontend/src/hooks/useAIAssistant.ts` | State management hook |
| `frontend/src/components/ProposalCard.tsx` | Proposal display + action buttons |
| `frontend/tests/fixtures/mock-ai-proxy.mjs` | Mock proxy for Playwright E2E tests |

---

## Troubleshooting

### "API key missing or invalid — check AI settings" (503)

The proxy is running but either:
- `OPENAI_API_KEY` is not set in the proxy's environment
- The API key is invalid or revoked

Fix: restart the proxy with a valid key:
```bash
OPENAI_API_KEY=sk-... just ai-proxy
```

### "Network error" / proxy unreachable

The frontend cannot reach the proxy. Common causes:
- The proxy is not running — start it with `just ai-proxy`
- The proxy is running on a different port — check that `frontend/src/services/ai-settings.ts` has the correct URL (default: `http://localhost:11435`)
- CORS origin mismatch — the proxy's `ALLOWED_ORIGINS` env var must include `http://localhost:5173`

### "AI panel doesn't open"

Ensure WASM is loaded — the panel requires `window.get_scene_snapshot` and `window.dispatch_command` to be available. Wait for the canvas to finish loading before clicking ✨ AI.

### Slow AI responses

- Large scenes hit the token limit faster. The proxy truncates scene context automatically (`TOKEN_THRESHOLD`, default: 10,000 tokens).
- Try more specific prompts to reduce scene description overhead.
- For local development without API costs, use Ollama instead of OpenAI.

### Playwright E2E tests fail

Ensure the mock proxy is running on port 11436:

```bash
node frontend/tests/fixtures/mock-ai-proxy.mjs
# In another terminal:
cd frontend && npx playwright test ai-assisted-editing.spec.ts
```

The `playwright.config.ts` starts both the mock proxy and Vite automatically when running tests via `just test` or `npx playwright test`.

---

## Running Tests

```bash
# Run the Rust proxy unit tests
just ai-proxy-test

# Run all Playwright E2E tests (includes AI-assisted tests)
just test

# Run only the AI-assisted editing tests
just test-one ai-assisted-editing.spec.ts

# Run with visible browser
just test-headed
```
