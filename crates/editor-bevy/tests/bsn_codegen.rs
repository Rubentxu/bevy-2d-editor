//! Integration tests for `bsn_codegen` — covers spec scenarios S1–S8.

use editor_bevy::{
    bsn_codegen::{emit_bsn_source, emit_bsn_source_from_document},
    bsn_ir_from_scene_asset,
    scene_asset::{
        LocalId, RelationshipKind, SceneAssetDocument, SceneAssetEntity, SceneAssetRelationship,
        SceneAssetRole,
    },
};
use editor_model::ComponentInstance;
use serde_json::json;

// ─── Test helpers ─────────────────────────────────────────────────────────────

fn mk_local_id(id: &str) -> LocalId {
    LocalId::new(id)
}

fn make_scene_asset(
    entities: Vec<SceneAssetEntity>,
    relationships: Vec<SceneAssetRelationship>,
) -> SceneAssetDocument {
    SceneAssetDocument {
        layers: vec![],
        asset_id: "test_asset".to_string(),
        logical_path: "test/level".to_string(),
        role: SceneAssetRole::Level,
        version: 1,
        entities,
        relationships,
        exposed_properties: vec![],
        metadata: Default::default(),
    }
}

fn scene_entity(
    local_id_val: &str,
    name: &str,
    components: Vec<ComponentInstance>,
) -> SceneAssetEntity {
    SceneAssetEntity {
        local_id: mk_local_id(local_id_val),
        local_path: format!("root/{}", name),
        name: name.to_string(),
        components,
    }
}

fn name_comp(name: &str) -> ComponentInstance {
    ComponentInstance {
        type_id: "editor.Name".to_string(),
        values: json!({ "name": name }),
    }
}

fn transform_comp(tx: f32, ty: f32, rot: f32, sx: f32, sy: f32) -> ComponentInstance {
    ComponentInstance {
        type_id: "editor.Transform2D".to_string(),
        values: json!({
            "translation": { "x": tx, "y": ty },
            "rotation": rot,
            "scale": { "x": sx, "y": sy },
        }),
    }
}

fn sprite_comp(asset: &str, r: f32, g: f32, b: f32, a: f32, anchor: &str) -> ComponentInstance {
    ComponentInstance {
        type_id: "editor.Sprite2D".to_string(),
        values: json!({
            "asset": asset,
            "color": { "r": r, "g": g, "b": b, "a": a },
            "anchor": anchor,
        }),
    }
}

fn visible_comp() -> ComponentInstance {
    ComponentInstance {
        type_id: "editor.Visible".to_string(),
        values: json!({ "visible": true }),
    }
}

fn locked_comp() -> ComponentInstance {
    ComponentInstance {
        type_id: "editor.Locked".to_string(),
        values: json!({ "locked": false }),
    }
}

fn game_comp(type_id: &str, values: serde_json::Value) -> ComponentInstance {
    ComponentInstance {
        type_id: type_id.to_string(),
        values,
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────────────

/// S1: single root with Name + Transform + Sprite → bsn_list![ bsn!{...} ] shape
#[test]
fn bsn_codegen_roundtrip_minimal_scene() {
    let doc = make_scene_asset(
        vec![scene_entity(
            "player_01",
            "Player",
            vec![
                name_comp("Player"),
                transform_comp(0.0, 0.0, 0.0, 1.0, 1.0),
                sprite_comp("assets/player.png", 1.0, 0.0, 0.0, 1.0, "Center"),
            ],
        )],
        vec![],
    );

    let result = emit_bsn_source_from_document(&doc, "level_01");
    let src = &result.source;

    assert!(
        src.contains("use bevy::prelude::*;"),
        "missing prelude import"
    );
    assert!(
        src.contains("pub fn spawn_level_01(mut commands: Commands)"),
        "missing spawn function"
    );
    assert!(
        src.contains("commands.spawn_scene_list(bsn_list!["),
        "missing spawn_scene_list call"
    );
    assert!(src.contains("bsn!{"), "missing bsn! opener");
    assert!(src.contains("#player_01"), "missing entity identifier");
    assert!(src.contains("Name(\"Player\")"), "missing Name component");
    assert!(
        src.contains(
            "Transform { translation: Vec2::new(0, 0), rotation: 0, scale: Vec2::new(1, 1) }"
        ),
        "missing Transform"
    );
    assert!(src.contains("Sprite {"), "missing Sprite block");
    assert!(src.contains("]).unwrap();"), "missing unwrap");
    assert!(
        result.source.ends_with('\n'),
        "source must end with newline"
    );
}

/// S2: parent + child with RelationshipKind::Child → Children [ ... ] block
#[test]
fn bsn_codegen_with_children() {
    let doc = make_scene_asset(
        vec![
            scene_entity(
                "parent_01",
                "Player",
                vec![name_comp("Player"), transform_comp(0.0, 0.0, 0.0, 1.0, 1.0)],
            ),
            scene_entity("child_01", "Sword", vec![name_comp("Sword")]),
        ],
        vec![SceneAssetRelationship {
            from_local_id: mk_local_id("parent_01"),
            to_local_id: mk_local_id("child_01"),
            kind: RelationshipKind::Child,
            field_path: None,
        }],
    );

    let ir = bsn_ir_from_scene_asset(&doc);
    let result = emit_bsn_source(&ir, "level_01");
    let src = &result.source;

    assert!(src.contains("Children ["), "missing Children block");
    assert!(src.contains("#child_01"), "missing child identifier");
    assert!(src.contains("Name(\"Sword\")"), "missing child Name");
    assert!(
        !src.contains("commands.entity("),
        "Commands::spawn leaked into bsn! output"
    );
    assert!(
        !src.contains("add_child("),
        "add_child leaked into bsn! output"
    );
}

/// S3 + S4: Sprite asset as string literal + Anchor as Anchor(Vec2) sibling
#[test]
fn bsn_codegen_sprite_with_anchor() {
    let doc = make_scene_asset(
        vec![scene_entity(
            "player_01",
            "Player",
            vec![
                name_comp("Player"),
                sprite_comp("assets/player.png", 1.0, 0.0, 0.0, 1.0, "TopLeft"),
            ],
        )],
        vec![],
    );

    let result = emit_bsn_source_from_document(&doc, "level_01");
    let src = &result.source;

    // Asset as bare string, not Handle::new() or .to_string()
    assert!(
        src.contains("image: \"assets/player.png\""),
        "asset should be bare string literal"
    );
    assert!(!src.contains("Handle::new"), "no Handle::new wrapper");
    assert!(!src.contains("Handle<Image>"), "no Handle type");
    assert!(
        !src.contains(".to_string()"),
        "no .to_string() on asset path"
    );

    // Anchor as sibling Anchor(Vec2) component
    assert!(
        src.contains("Anchor(Vec2::new(-0.5, 0.5))"),
        "TopLeft anchor not emitted correctly"
    );
    assert!(
        !src.contains("Anchor::TOP_LEFT"),
        "no Anchor named constant"
    );
}

/// S5: editor.Visible / editor.Locked silently skipped, Transform emitted
#[test]
fn bsn_codegen_skips_editor_components() {
    let doc = make_scene_asset(
        vec![scene_entity(
            "entity_01",
            "E",
            vec![
                name_comp("E"),
                visible_comp(),
                locked_comp(),
                transform_comp(0.0, 0.0, 0.0, 1.0, 1.0),
            ],
        )],
        vec![],
    );

    let result = emit_bsn_source_from_document(&doc, "level_01");
    let src = &result.source;

    assert!(!src.contains("Visible"), "Visible should not appear");
    assert!(!src.contains("Locked"), "Locked should not appear");
    assert!(
        src.contains(
            "Transform { translation: Vec2::new(0, 0), rotation: 0, scale: Vec2::new(1, 1) }"
        ),
        "Transform should appear"
    );
    assert_eq!(
        result.warnings.len(),
        0,
        "no warnings for silently-skipped types"
    );
}

/// S7: empty scene → bsn_list![] with // Empty scene comment, no bsn! opener
#[test]
fn bsn_codegen_empty_scene() {
    let doc = make_scene_asset(vec![], vec![]);

    let result = emit_bsn_source_from_document(&doc, "level_01");

    let expected = concat!(
        "// ═══════════════════════════════════════════════════════════════════════════\n",
        "// ⚠️  AUTO-GENERATED — edits will be lost on next export\n",
        "// Bevy 0.19 | Generated by Bevy 2D Editor | BSN output\n",
        "// ═══════════════════════════════════════════════════════════════════════════\n",
        "\n",
        "use bevy::prelude::*;\n",
        "\n",
        "pub fn spawn_level_01(mut commands: Commands) {\n",
        "    commands.spawn_scene_list(bsn_list![\n",
        "        // Empty scene\n",
        "    ]).unwrap();\n",
        "}\n",
    );

    assert_eq!(result.source, expected);
    assert!(
        !result.source.contains("bsn!{"),
        "no bsn! block for empty scene"
    );
    assert!(result.warnings.is_empty(), "no warnings for empty scene");
}

/// S6: unknown (non-editor.*, non-game.*) component → warning + skipped; Name survives
#[test]
fn bsn_codegen_warns_on_unknown_component() {
    let doc = make_scene_asset(
        vec![scene_entity(
            "entity_01",
            "X",
            vec![
                name_comp("X"),
                game_comp("mystery.Bar", json!({ "baz": "x" })),
            ],
        )],
        vec![],
    );

    let result = emit_bsn_source_from_document(&doc, "level_01");
    let src = &result.source;

    // Unknown type not in output
    assert!(
        !src.contains("mystery.Bar"),
        "unknown mystery.Bar should not appear"
    );
    // Known component still present
    assert!(src.contains("Name(\"X\")"), "Name should still appear");
    // Warnings for unknown types
    assert_eq!(
        result.warnings.len(),
        1,
        "expected 1 warning for unknown type"
    );
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.message.contains("mystery.Bar")),
        "warning must reference mystery.Bar"
    );
}

/// S6 (defensive): game.* components are emitted as struct literals with no warning
#[test]
fn bsn_codegen_game_component_emitted_as_struct() {
    let doc = make_scene_asset(
        vec![scene_entity(
            "player_01",
            "Player",
            vec![
                name_comp("Player"),
                game_comp("game.Health", json!({ "hp": 100 })),
            ],
        )],
        vec![],
    );

    let result = emit_bsn_source_from_document(&doc, "level_01");
    assert!(
        result.warnings.is_empty(),
        "game.* must not warn, got: {:?}",
        result.warnings
    );
    assert!(
        result.source.contains("Health"),
        "game.Health must be emitted as struct"
    );
    assert!(
        result.source.contains("hp: 100"),
        "game.Health field must be emitted"
    );
}
