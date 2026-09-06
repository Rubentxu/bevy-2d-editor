# ADR-0054: Rig Agent Runtime Foundation — Transport Neutrality Addendum

## Status

Accepted — 2026-09-06

## Relationship to ADR-0027 and ADR-0043

**Extends [ADR-0027](./0027-rig-agentic-editor-architecture.md)** and **ADR-0043** by establishing that the `agent-runtime` crate must be transport-neutral. ADR-0027 selected Rig as the orchestration framework, and ADR-0043 established the capability-boundary model. This ADR adds the transport-neutrality constraint for the agent runtime foundation.

## Context

ADR-0027 chose Rig as the orchestration framework and outlined the `agent-runtime` crate architecture. ADR-0043 established that the agent runtime must talk to the editor through `editor-protocol` capability tools and must not bypass validation/approval policies.

However, neither ADR specifies the transport mechanism between the `ai-proxy` HTTP boundary and the `agent-runtime`. The initial implementation may assume an in-process or same-process transport, but the architecture must support transport replacement without redesigning the agent runtime or editor capability boundaries.

The transport-neutrality requirement ensures:

1. **Swapability**: The transport layer can be replaced (e.g., from in-process to HTTP to WASM message-passing) without modifying the agent runtime or editor capability interfaces.
2. **Testability**: The agent runtime can be tested with a mock transport without requiring a live editor session.
3. **Polyglot readiness**: A future multi-language agent runtime (e.g., Python for ML tasks) can reuse the same transport protocol.

## Decision

The `agent-runtime` crate is designed as a **transport-agnostic core**. The transport layer is explicitly decoupled from the agent runtime and is injected via a trait or configuration at runtime.

```text
ai-proxy (HTTP boundary)
       │
       │  ← transport plug (e.g., in-process channel, HTTP client, WASM bridge)
       ▼
agent-runtime (transport-agnostic core)
       │
       │  ← typed capability tools (editor-protocol)
       ▼
editor-protocol / EditorSession
```

### Specific constraints

1. **No direct HTTP server imports in `agent-runtime`**. The runtime must not import `axum`, `actix`, or any HTTP server crate. If a transport requires an HTTP server, that server lives in `ai-proxy` or a separate transport adapter crate.

2. **Transport defined by a trait**. The transport between `ai-proxy` and `agent-runtime` is defined by a `Transport` trait in `agent-runtime`:
   ```rust
   pub trait Transport: Send + Sync {
       type Error: std::error::Error + Send + Sync + 'static;
       async fn send(&self, msg: AgentMessage) -> Result<AgentResponse, Self::Error>;
   }
   ```

3. **Built-in in-process transport**. The `agent-runtime` crate ships a built-in in-process channel transport for same-process use cases. This transport is used by default in the editor context.

4. **The `ai-proxy` crate is banned** from the workspace per OQ-D1. Any HTTP transport adapter must live in a separate crate that `ai-proxy` depends on, not in `agent-runtime` itself.

5. **`editor-core` is a tripwire**. The crate name `editor-core` does not exist in the workspace (it was renamed to `editor-bevy` and `editor-wasm` per ADR-0030). The `deny.toml` entry for `editor-core` is a tripwire to catch any accidental references and produce a warning.

## Architecture

### Transport trait (pseudocode)

```rust
// In agent-runtime transport module
use async_trait::async_trait;

#[async_trait]
pub trait AgentTransport: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Send a message to the agent runtime and wait for a response.
    async fn roundtrip(&self, request: AgentRequest) -> Result<AgentResponse, Self::Error>;

    /// Send a message without waiting for a response (fire-and-forget).
    async fn send(&self, request: AgentRequest) -> Result<(), Self::Error>;
}
```

### Built-in transports

1. **InProcessChannel**: A `mpsc` channel-based transport for same-process communication. This is the default for the editor context.

2. **WasmBridge** (future): A `postMessage`-based transport for WASM message passing between the editor frontend and a secondary agent runtime worker.

### Editor capability boundary (unchanged from ADR-0043)

The agent runtime accesses editor capabilities through `editor-protocol` tool ports. The transport layer does not affect this boundary:

```text
agent-runtime
  └─► editor-protocol tool ports (EditorCapabilityPort trait)
        └─► EditorSession / ChangeSet approval pipeline
```

## Consequences

### Positive

- The agent runtime can be tested in isolation with a mock transport.
- The transport can be upgraded or replaced without touching the agent runtime or editor.
- A future Python or Go agent runtime can implement the same `Transport` trait.

### Negative

- An extra trait abstraction adds indirection. In the common in-process case, this is a zero-cost abstraction at runtime (compiler can optimize through the trait), but it does add compile-time complexity.

### Neutral

- The `ai-proxy` ban means HTTP-specific transport adapters must live in a separate adapter crate. This is consistent with the ADR-0043 capability boundary intent.

## References

- [ADR-0027: Rig-Based Agent Runtime](./0027-rig-agentic-editor-architecture.md)
- [ADR-0043: Agent Runtime Capability Boundary](./0043-agent-runtime-capability-boundary.md)
- [OQ-D1: Non-existent crate reference](./application-stabilization-roadmap.md) — the spec references `editor-core` which does not exist; the tripwire in `deny.toml` catches accidental references.
