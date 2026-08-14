# Suggested Technical Spikes

Spikes answer bounded questions and end with a decision/report, not production scope creep.

## SPIKE-001 Rust ↔ TypeScript contract generation

Compare practical options for shared protocol types with WASM. Criteria: serde compatibility, enums/errors, build integration, generated code reviewability, browser bundle impact.

## SPIKE-002 UUIDv7 vs ULID generator

Benchmark mature Rust crates on native + wasm32, monotonic burst behavior, serialization size and dependency weight.

## SPIKE-003 Filesystem access mode

Evaluate browser File System Access API versus Tauri/native companion versus both behind `ProjectStore`. Include Bazzite/Linux workflow and security sandboxing.

## SPIKE-004 Lossless/semantic BSN representation

Test current Bevy BSN syntax/APIs against representative Scene Assets, unsupported editor metadata and write-back requirements.

## SPIKE-005 Transaction semantics across multiple files

Prototype prepare/write-temp/rename atomicity for OPFS and filesystem. Define compensating behavior where true atomicity cannot be guaranteed.

## SPIKE-006 Large-project UI performance

Build synthetic 1k/10k entity projects and profile hierarchy, inspector, search, viewport and serialization costs.

## SPIKE-007 Runtime trace overhead

Measure Logic Bricks activation/provenance tracing with bounded ring buffers and optional diagnostic channels.

## SPIKE-008 Extension execution model

After internal SDK consumers exist, compare compiled-in Rust extensions, WASM component/plugin approaches and external-process capability clients. Do not select public ABI before evidence.
