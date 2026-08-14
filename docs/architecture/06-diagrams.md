# Architecture Diagrams

## C4 — System context

```mermaid
flowchart LR
  DEV[Developer / Level Designer / Technical Artist]
  WORKBENCH[Bevy 2D Workbench]
  REPO[Bevy Game Repository]
  LLM[Local / Remote LLM Providers]
  TOOLS[External Authoring Tools
Aseprite / LDtk / Tiled]
  BEVY[Bevy Runtime / Cargo Toolchain]

  DEV --> WORKBENCH
  WORKBENCH <--> REPO
  WORKBENCH <--> TOOLS
  WORKBENCH --> BEVY
  WORKBENCH <--> LLM
```

## C4 — Containers

```mermaid
flowchart TB
  UI[React Editor Shell]
  WASM[editor-wasm]
  APP[editor-application]
  MODEL[editor-model]
  BEVY[editor-bevy]
  WEB[editor-storage-web]
  FS[editor-storage-fs]
  PROTOCOL[editor-protocol]
  AGENT[agent-runtime]
  PROXY[ai-proxy]

  UI --> WASM
  WASM --> APP
  APP --> MODEL
  WASM --> BEVY
  WASM --> WEB
  WASM --> PROTOCOL
  FS --> APP
  AGENT --> PROTOCOL
  PROXY --> AGENT
```

## Change flow

```mermaid
sequenceDiagram
  participant U as Human/Recipe/Agent/Importer
  participant C as Capability
  participant T as TransactionKernel
  participant V as Validation
  participant W as Change Workbench
  participant S as EditorSession
  participant R as Runtime Adapter

  U->>C: request intent
  C->>T: build ChangeSet
  T->>V: preflight
  V-->>T: issues + risk
  T-->>W: proposed semantic diff
  W-->>T: approve / reject / partial selection
  T->>S: atomic apply
  S-->>T: inverses + effects
  T->>R: refresh/hot-reload effects
  R-->>T: runtime verification
  T-->>W: result + rollback handle
```

## Definition / Instance / Override

```mermaid
flowchart LR
  DEF[Scene Asset Definition]
  I1[Scene Instance A]
  I2[Scene Instance B]
  OV1[Overrides A]
  OV2[Overrides B]

  DEF --> I1
  DEF --> I2
  OV1 --> I1
  OV2 --> I2
```

## World workspace

```mermaid
flowchart LR
  W[WorldDocument]
  A[Village Level]
  B[Forest Level]
  C[Cave Level]
  D[Boss Level]
  W --> A
  W --> B
  W --> C
  W --> D
  A -- east --> B
  A -- down --> C
  B -- down --> D
  C -- east --> D
```

## Runtime causality graph

```mermaid
flowchart LR
  WORLD[World/Level]
  ASSET[Scene Asset]
  INST[Scene Instance]
  COMP[Component]
  LOGIC[Logic Graph]
  RUNTIME[Runtime Entity]
  SOURCE[Rust Source]
  CHANGE[ChangeSet / History]

  WORLD --> INST
  ASSET --> INST
  INST --> COMP
  COMP --> RUNTIME
  LOGIC --> RUNTIME
  SOURCE --> COMP
  CHANGE --> ASSET
  CHANGE --> INST
  CHANGE --> SOURCE
```

## Agent architecture

```mermaid
flowchart TB
  M[Manager Agent]
  SA[Scene Specialist]
  AA[Asset Specialist]
  LA[Logic Specialist]
  CA[Code Specialist]
  VA[Validation Specialist]
  RA[Runtime Specialist]
  REG[Typed Capability Tool Registry]
  CW[Change Workbench]

  M --> SA
  M --> AA
  M --> LA
  M --> CA
  M --> VA
  M --> RA
  SA --> REG
  AA --> REG
  LA --> REG
  CA --> REG
  VA --> REG
  RA --> REG
  REG --> CW
```
