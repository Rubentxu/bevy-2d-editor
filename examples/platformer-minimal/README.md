# platformer-minimal — Canonical v1.0 Sample Game

> **Purpose:** The smallest playable 2D game that exercises every editor feature
> the v1.0 product gates require evidence for. Not a tutorial — a *canary*.
> If this loads in the editor and runs in Bevy, the editor pipeline is proven
> end-to-end.
>
> **Status:** P1 deliverable of v1.0-stabilization cycle (2026-09-06).
> See `docs/v1.0-stabilization-evidence-map.md` §7 for context.

---

## What this sample demonstrates

| # | Editor feature | Evidence in this sample |
|---|---|---|
| 1 | **Project root** (`project.json`) | `project.json` declares version, name, scenes, schemas, scene assets. |
| 2 | **Scenes** (`SceneDocument`) | `scenes/main.scene.json` — 1 scene with 4 entities. |
| 3 | **Scene Instances** (placed scene assets) | `scenes/main.scene.json` places player, enemy, ground, pickup as Scene Instances. |
| 4 | **Component Overrides** (ADR-0008) | Enemy and pickup instances override the base asset's `editor.Transform2D` for placement. |
| 5 | **Custom schemas** (Component Schema Registry) | `schemas/game.PlayerController.schema.json`, `schemas/game.EnemyPatrol.schema.json`. |
| 6 | **Built-in schemas** | `editor.Name`, `editor.Transform2D`, `editor.Sprite2D`, `editor.Visible`. |
| 7 | **Reusable Scene Assets** (ADR-0008) | `scene-assets/characters/{player,enemy}.actor.json`, `scene-assets/environment/ground.fragment.json`, `scene-assets/effects/pickup.actor.json`. |
| 8 | **Scene Asset Roles** | `actor`, `fragment`, `effect` (covers 3 of 7 roles). |
| 9 | **Logic Bricks (LogicGraphAsset)** | `logic-graphs/contact-death.logic.json` — sensor→controller→actuator graph. |
| 10 | **BSN export** (`.bsn` file output) | Each scene asset has a corresponding `export/*.bsn` file produced by `emit_bsn_source_from_document`. |
| 11 | **OPFS round-trip** | All files load via `load_project()` after page reload (mirrors `engine.spec.ts:744`). |
| 12 | **Schema type-id uniqueness** | Schemas use namespaced `game.*` type_ids (rule 6 of docs-check). |

---

## File tree

```
examples/platformer-minimal/
├── README.md                                       # this file
├── project.json                                    # editor project root
├── schemas/
│   ├── game.PlayerController.schema.json
│   └── game.EnemyPatrol.schema.json
├── scenes/
│   └── main.scene.json                             # runtime SceneDocument
├── scene-assets/
│   ├── characters/
│   │   ├── player.actor.json                       # reusable player asset
│   │   └── enemy.actor.json                        # reusable enemy asset
│   ├── environment/
│   │   └── ground.fragment.json                    # reusable ground fragment
│   └── effects/
│       └── pickup.actor.json                       # reusable pickup asset
├── logic-graphs/
│   └── contact-death.logic.json                    # Logic Bricks graph
└── export/
    ├── player.bsn                                  # BSN export of player.actor
    ├── enemy.bsn                                   # BSN export of enemy.actor
    ├── ground.bsn                                  # BSN export of ground.fragment
    └── pickup.bsn                                  # BSN export of pickup.actor
```

When the editor loads this project (via OPFS or via the e2e spec), the
resulting OPFS tree after `load_project()` mirrors the structure above.

---

## How to load this sample

### Option A — manually into OPFS (development)

```bash
# 1. Build the WASM bundle and serve the editor
cd frontend && npm ci && npm run dev

# 2. Open http://localhost:5173 in a Chromium browser.

# 3. In the browser DevTools console, populate OPFS via the bridge:
#    (See `frontend/tests/e2e-game-creation.spec.ts` for the canonical script.)
```

### Option B — automated via Playwright

```bash
cd frontend
npx playwright test --config=playwright.smoke.config.ts e2e-game-creation.spec.ts
```

The Playwright spec:
1. Mounts OPFS via the `opfs_*` bridges.
2. Calls `window.load_project()`.
3. Asserts entities render in the Hierarchy panel.
4. Asserts Scene Asset catalog lists `characters/player`, `characters/enemy`,
   `environment/ground`, `effects/pickup`.
5. Asserts schemas `game.PlayerController`, `game.EnemyPatrol` are registered.
6. Asserts the Logic Graph `logic_graphs/contact-death.logic.json` is bound.

### Option C — Bevy runtime harness (future iteration)

The `export/*.bsn` files are produced by `crates/editor-bevy/src/bsn_export.rs`
via `emit_bsn_source_from_document`. The Bevy harness that consumes them is
**not yet in this repo** — it is roadmap item P3 of the Evidence Map (a
follow-up to P1). Until that ships, evidence of runtime playability is
limited to the editor's `enter_play_mode()` preview (covered by
`runtime-preview-v2.spec.ts`).

---

## Game design (intentionally minimal)

**World**: A 2D platformer slice, 800×600 units. Ground spans the bottom.
**Player**: Blue square, `editor.Sprite2D` with color `(0.2, 0.4, 0.9, 1.0)`.
Driven by `game.PlayerController` schema (custom fields: `speed`, `jump_force`).
**Enemy**: Red square, `game.EnemyPatrol` (custom fields: `speed`, `patrol_range`).
**Pickup**: Yellow square (killing-on-touch for simplicity — a real game would
collect on touch).
**Logic graph**: `contact-death.logic.json` — Sensor: "OnContactEnter" → Controller: "Branch if other is EnemyTag" → Actuator: "Destroy self". This is the
minimum viable behavior graph.

The game is **deliberately ugly** — no art, no animation, no physics. Its job
is to exercise the editor pipeline, not to ship as a real game.

---

## Reproducing this sample in the editor (manual authoring)

If you want to recreate this sample by hand in the editor (instead of loading
the JSON files directly), here is the sequence:

1. **Open the editor.** Click "New Project" → name it `platformer-minimal`.
2. **Register schemas.** Open the Schema Authoring panel, register:
   - `game.PlayerController` with fields `speed: f32 = 200`, `jump_force: f32 = 400`.
   - `game.EnemyPatrol` with fields `speed: f32 = 60`, `patrol_range: f32 = 150`.
3. **Create scene assets.** In the Scene Asset Browser:
   - `characters/player` (role: actor) — entity with `editor.Name`="Player",
     `editor.Transform2D`, `editor.Sprite2D`, `game.PlayerController`.
   - `characters/enemy` (role: actor) — entity with `editor.Name`="Enemy",
     `editor.Transform2D`, `editor.Sprite2D` (red), `game.EnemyPatrol`.
   - `environment/ground` (role: fragment) — entity with
     `editor.Transform2D` (large scale), `editor.Sprite2D` (green).
   - `effects/pickup` (role: actor) — entity with `editor.Name`="Pickup",
     `editor.Transform2D`, `editor.Sprite2D` (yellow).
4. **Create the main scene.** Open `scenes/main.scene.json`:
   - Place 4 Scene Instances (player, enemy, ground, pickup) using the
     Project Asset Browser → "Place Instance".
   - Override each instance's `editor.Transform2D` for placement.
5. **Create the logic graph.** In the Logic Graph panel, create
   `logic_graphs/contact-death.logic.json`:
   - Sensor: OnContactEnter
   - Controller: Branch (if other has EnemyTag → destroy self)
   - Actuator: DestroyEntity
6. **Bind the logic graph** to the player instance in the main scene.
7. **Save.** Click File → Save Project.
8. **Export.** Click Tools → Export Scene Asset → `.bsn`. (Per asset.)

The resulting OPFS tree matches the file tree above.

---

## Conventions used

- **Stable IDs:** hand-assigned for reproducibility (`ent_player`, `ent_enemy`,
  `asset_player`, `inst_player_main`, etc.).
- **Naming:** `game.*` namespace for user-authored schemas (avoiding collision
  with built-in `editor.*` schemas).
- **Roles:** chosen from the 7 documented roles (Actor, Fragment, Screen,
  Level, Ui, Effect, Logic). Three used here.
- **No stubs:** every component has real values; no placeholders.

---

## Future iterations (tracked in Evidence Map)

- **P3 (Evidence Map §7):** Bevy runtime harness that consumes `export/*.bsn`
  and runs the game. Will be a new `crates/examples-bevy-harness/` workspace
  crate.
- **P6 (Evidence Map §7):** `CONTRIBUTING.md` documenting how to author a
  canonical sample (generalising this README into a recipe).
- **UX onboarding (Evidence Map P7):** the editor should detect no-project
  state and offer this sample as the "first game" tutorial.
