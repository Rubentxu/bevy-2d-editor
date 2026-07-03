# ADR-0013: Build & Run Loop — Enhanced Preview Mode for v1

## Status
Accepted (2026-07-03)

## Context

The Bevy 2D Editor's Hito 4 roadmap describes a `build-and-run-loop` that lets users "press Play" and see their game run. The original vision assumed the browser could compile user-authored Rust code.

Investigation (2026-07-03) proved this infeasible:
- `rustc.wasm` is 9-star experimental, ~100MB download, single-threaded, 10-30x slower. **Cannot compile Bevy's 200+ dependency tree in a browser.**
- Remote build server (POST source → server compile) adds network dependency, 60-120s cold compile latency, and security concerns.
- "Running a game" from a browser-based editor is a pattern shared by Construct, GDevelop, and Flowlab — all of which interpret visual logic at runtime without native compilation.

The browser editor cannot be a Bevy compiler. It can be a **game engine with a visual editor**, like the tools above.

## Decision

**v1: Enhanced Preview Mode.** The "Play" button switches the existing Bevy preview instance into a persistent play mode rather than a one-shot scene rebuild.

The Bevy `App` already runs continuously in the browser. `rebuild_preview_world()` despawns/respawns entities from `SceneDocument` JSON on dirty. Logic Bricks dispatch (`logic_evaluation_system`) already runs compiled `NodeEvaluator` structs in the Bevy `Update` loop.

"Run game" = take this existing infrastructure and add:
1. A **Play/Stop toggle** that freezes editor input and hands keyboard/mouse to the game world.
2. **Persistent game state** across multiple play sessions (not reset on every scene change).
3. **Game overlay** showing FPS, preview metrics, and a "Stop" button.
4. **Input capture** that routes game inputs (WASD, mouse) to the game world, not the editor.

Logic Bricks provides the runtime behavior vocabulary (jump, collision, health, timers, proximity). This is sufficient for v1 platformer-quality 2D games.

## Future (v2+)

Remote build server remains the correct path for full Rust compilation. When Bevy ships a stable WASM compilation story or a cloud build service becomes viable, Approach 1 from the explore becomes the right choice.

## Consequences

### Positive
- Instant feedback (no compile step)
- Works fully offline
- Leverages all existing infrastructure (preview world, logic dispatch, command bus)
- Aligned with Construct/GDevelop/Flowlab mental model
- No server infrastructure needed

### Negative
- User-authored Rust code cannot run in v1 (only Logic Bricks)
- Scope of "game" is limited to what Logic Bricks can express
- Input routing adds complexity to the editor UX

## Scope Discipline

v1 covers ONLY the enhanced preview loop. The following are explicitly **out of scope** and deferred to v2:
- Remote build server compilation
- Custom Rust `NodeEvaluator` plugins by users
- Multi-scene game flow (start screen → level → end screen)
- Audio/SFX runtime
- Persistent game save states

## References

- ROADMAP.md § Hito 4 Order 4
- Explore report: `sddk/build-and-run-loop/explore-report.md`
- ADR-0002: Editor Modes (editor/preview/game)
- `crates/editor-core/src/lib.rs` — `start_engine()`, `rebuild_preview_world()`
- `crates/editor-core/src/logic_dispatch.rs` — `logic_evaluation_system`
