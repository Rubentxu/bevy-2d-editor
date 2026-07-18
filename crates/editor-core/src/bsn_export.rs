//! BSN file export — serializes a `SceneAssetDocument` to `.bsn` text.
//!
//! This is the **output** path for the `.bsn` file I/O capability. Import
//! (`.bsn` text → `SceneAssetDocument` / `BsnIr`) is deferred to a future
//! slice. The trait abstraction in this module is the swap point for
//! Bevy's official writer (PR #23639) when it stabilizes; see ADR-0010
//! for the migration path.
//!
//! Per ADR-0005 §BSN IR: the export goes through `BsnIr` (one-way, lossy
//! projection of the editor source-of-truth) and then through a private
//! `emit_bsn_text` that produces raw `.bsn` syntax — no Rust
//! `commands.spawn_scene_list(...)` wrapper and no Rust tuple commas
//! inside `Children`.

use std::fmt::Write as FmtWrite;

use crate::bsn_ir::{bsn_ir_from_scene_asset, BsnIr, BsnIrNode};
use crate::code_export::CodeGenResult;
use crate::dynamic_scene::{anchor_str_to_normalized_offset, ExportWarning};
use crate::scene_asset::SceneAssetDocument;

/// Errors that can occur when exporting a `SceneAssetDocument` to `.bsn` text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BsnExportError {
    /// The document has no entities, so there is nothing to export.
    EmptyScene,
    /// The IR contains a shape we do not support (reserved for future use).
    UnsupportedShape(String),
    /// Generic IO error placeholder (e.g., file write). The current slice
    /// returns text, not a file path, so this variant is reserved.
    IoError(String),
}

impl std::fmt::Display for BsnExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BsnExportError::EmptyScene => f.write_str("SceneAssetDocument has no entities; nothing to export"),
            BsnExportError::UnsupportedShape(s) => {
                write!(f, "Unsupported BSN shape: {}", s)
            }
            BsnExportError::IoError(s) => write!(f, "BSN export IO error: {}", s),
        }
    }
}

impl std::error::Error for BsnExportError {}

/// The swappable contract for `.bsn` file export.
///
/// Implementations of `BsnExporter` take a `SceneAssetDocument` (the editor
/// source of truth) and produce the raw `.bsn` text suitable for hand-off
/// to a Bevy runtime. New Bevy-native impls can land without disturbing
/// callers.
pub trait BsnExporter: Send + Sync {
    /// Serialize the document to `.bsn` text.
    fn export_to_bsn_text(
        &self,
        doc: &SceneAssetDocument,
    ) -> Result<String, BsnExportError>;
}

// ─────────────────────────────────────────────────────────────────────────────
// EditorCoreBsnExporter (impl 1)
// ─────────────────────────────────────────────────────────────────────────────

/// The editor's own `.bsn` writer. Builds the `BsnIr` from the document and
/// emits raw `.bsn` text. This is the only working impl today.
pub struct EditorCoreBsnExporter;

impl BsnExporter for EditorCoreBsnExporter {
    fn export_to_bsn_text(
        &self,
        doc: &SceneAssetDocument,
    ) -> Result<String, BsnExportError> {
        // Reject Logic role — BSN export is for scene assets only
        if matches!(doc.role, crate::scene_asset::SceneAssetRole::Logic) {
            return Err(BsnExportError::UnsupportedShape(
                "logic role is not exported to .bsn".into(),
            ));
        }
        if doc.entities.is_empty() {
            return Err(BsnExportError::EmptyScene);
        }
        let ir = bsn_ir_from_scene_asset(doc);
        let mut warnings: Vec<ExportWarning> = Vec::new();
        let text = emit_bsn_text(&ir, &mut warnings);
        Ok(text)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BevyBsnExporter (impl 2 — placeholder for future Bevy-native writer)
// ─────────────────────────────────────────────────────────────────────────────

/// Placeholder for the future Bevy-native `.bsn` writer. Will be implemented
/// once Bevy PR #23639 (BSN scene writer) and PR #23648 (BSN asset catalog)
/// land on Bevy main. Until then, this struct compiles and has the same
/// type as `EditorCoreBsnExporter` (both implement `BsnExporter`).
///
/// Migration path (ADR-0010): when Bevy writer lands, this struct will hold a
/// `bevy_scene2::bsn_writer::BsnWriter` (or equivalent) and forward the call.
pub struct BevyBsnExporter;

// TODO: implement once Bevy PR #23639 lands:
// impl BsnExporter for BevyBsnExporter { ... }

// ─────────────────────────────────────────────────────────────────────────────
// Public convenience API
// ─────────────────────────────────────────────────────────────────────────────

/// Export a `SceneAssetDocument` to `.bsn` text using the editor's own
/// `EditorCoreBsnExporter`. Equivalent to `EditorCoreBsnExporter::export_to_bsn_text`.
pub fn export_to_bsn_text(doc: &SceneAssetDocument) -> Result<String, BsnExportError> {
    EditorCoreBsnExporter.export_to_bsn_text(doc)
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal: raw `.bsn` emitter
// ─────────────────────────────────────────────────────────────────────────────

/// Emit raw `.bsn` text (no Rust wrapper) from a `BsnIr`. The output is
/// valid `.bsn` syntax that can be loaded by Bevy's (forthcoming) `.bsn`
/// asset loader.
fn emit_bsn_text(ir: &BsnIr, _warnings: &mut Vec<ExportWarning>) -> String {
    let mut out = String::new();

    if ir.scene_root.components.is_empty()
        && ir.scene_root.children.is_empty()
        && ir.scene_root.identifier == "empty"
    {
        return String::new();
    }

    emit_bsn_node(&mut out, &ir.scene_root, 0, _warnings);
    out
}

/// Recursively emit a single `bsn!{ ... }` block for a `BsnIrNode`. The
/// output is `.bsn`-native (no Rust tuple commas, no `,` after `Children`
/// items) so the file is a valid `.bsn` asset, not a Rust source.
fn emit_bsn_node(out: &mut String, node: &BsnIrNode, indent: usize, warnings: &mut Vec<ExportWarning>) {
    let indent_str = "    ".repeat(indent);

    let _ = writeln!(out, "{}bsn!{{", indent_str);
    let _ = writeln!(out, "{}#{}", indent_str, node.identifier);

    for (type_id, values) in &node.components {
        emit_component(out, type_id, values, indent + 1, warnings);
    }

    if !node.children.is_empty() {
        let _ = writeln!(out, "{}Children [", indent_str);
        for child in &node.children {
            emit_bsn_node(out, child, indent + 1, warnings);
        }
        let _ = writeln!(out, "{}]", indent_str);
    }

    let _ = writeln!(out, "{}}}", indent_str);
}

/// Emit a single component inside a `bsn!{ ... }` block. The format here
/// is the `.bsn` syntax (e.g. `Name("...")`, `Transform { ... }`).
fn emit_component(
    out: &mut String,
    type_id: &str,
    values: &serde_json::Value,
    indent: usize,
    warnings: &mut Vec<ExportWarning>,
) {
    let indent_str = "    ".repeat(indent);

    match type_id {
        t if is_editor_only_type(t) => {
            // Silently skip editor-only types: editor.Visible, editor.Locked
        }
        "editor.Name" => {
            let name = values.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let _ = writeln!(out, "{}Name(\"{}\")", indent_str, escape_string(name));
        }
        "editor.Transform2D" => {
            let tx = values
                .get("translation")
                .and_then(|v| v.get("x"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;
            let ty = values
                .get("translation")
                .and_then(|v| v.get("y"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;
            let rot = values
                .get("rotation")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;
            let sx = values
                .get("scale")
                .and_then(|v| v.get("x"))
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0) as f32;
            let sy = values
                .get("scale")
                .and_then(|v| v.get("y"))
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0) as f32;
            writeln!(
                out,
                "{}Transform {{ translation: Vec2::new({}, {}), rotation: {}, scale: Vec2::new({}, {}) }}",
                indent_str, tx, ty, rot, sx, sy
            )
            .unwrap();
        }
        "editor.Sprite2D" => {
            let asset = values.get("asset").and_then(|v| v.as_str()).unwrap_or("");
            let r = values
                .get("color")
                .and_then(|v| v.get("r"))
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0) as f32;
            let g = values
                .get("color")
                .and_then(|v| v.get("g"))
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0) as f32;
            let b = values
                .get("color")
                .and_then(|v| v.get("b"))
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0) as f32;
            let a = values
                .get("color")
                .and_then(|v| v.get("a"))
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0) as f32;
            let anchor_str = values
                .get("anchor")
                .and_then(|v| v.as_str())
                .unwrap_or("Center");

            writeln!(
                out,
                "{}Sprite {{ image: \"{}\", color: Color::srgba({}, {}, {}, {}) }}",
                indent_str,
                escape_string(asset),
                r, g, b, a
            )
            .unwrap();

            let (ax, ay) = anchor_str_to_normalized_offset(anchor_str);
            if ax != 0.0 || ay != 0.0 {
                let _ = writeln!(out, "{}Anchor(Vec2::new({}, {}))", indent_str, ax, ay).unwrap();
            }
        }
        t if is_user_type(t) => {
            let struct_name = pascal_case_struct_name(t);
            let fields = emit_struct_fields(values, warnings);
            let _ = writeln!(out, "{}{} {{ {} }}", indent_str, struct_name, fields);
        }
        _ => {
            // Unknown type — emit a placeholder comment
            writeln!(
                out,
                "{}// unknown component type: {}",
                indent_str, type_id
            )
            .unwrap();
        }
    }
}

fn is_editor_only_type(type_id: &str) -> bool {
    matches!(type_id, "editor.Visible" | "editor.Locked")
}

fn is_user_type(type_id: &str) -> bool {
    type_id.starts_with("game.")
}

fn pascal_case_struct_name(type_id: &str) -> String {
    let parts: Vec<&str> = type_id.split('.').collect();
    let last = parts.last().copied().unwrap_or("");
    let mut out = String::new();
    let mut upper_next = true;
    for ch in last.chars() {
        if ch == '_' || ch == '-' {
            upper_next = true;
            continue;
        }
        if upper_next {
            out.extend(ch.to_uppercase());
            upper_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}

fn emit_struct_fields(values: &serde_json::Value, _warnings: &mut Vec<ExportWarning>) -> String {
    let _ = _warnings; // currently unused; reserved for future warning emission
    let mut parts: Vec<String> = Vec::new();
    if let serde_json::Value::Object(obj) = values {
        for (k, v) in obj {
            let lit = format_bsn_literal(v);
            if !lit.is_empty() {
                parts.push(format!("{}: {}", k, lit));
            }
        }
    }
    parts.join(", ")
}

fn format_bsn_literal(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("\"{}\"", escape_string(s)),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => String::new(),
    }
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ─────────────────────────────────────────────────────────────────────────────
// Public convenience: convenience alias for code_export
// ─────────────────────────────────────────────────────────────────────────────

/// Export and return both the text and any warnings. Currently a thin
/// wrapper that returns an empty warning list (the IR is lossy by design,
/// but most warnings are not generated at this level).
pub fn export_to_bsn_text_with_warnings(
    doc: &SceneAssetDocument,
) -> Result<CodeGenResult, BsnExportError> {
    let text = export_to_bsn_text(doc)?;
    Ok(CodeGenResult {
        source: text,
        warnings: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_asset::{
        LocalId, RelationshipKind, SceneAssetDocument, SceneAssetEntity, SceneAssetMetadata,
    };
    use std::collections::BTreeMap;

    fn make_doc(entities: Vec<SceneAssetEntity>) -> SceneAssetDocument {
        SceneAssetDocument {
            asset_id: "asset-1".to_string(),
            logical_path: "assets/test".to_string(),
            role: crate::scene_asset::SceneAssetRole::Actor,
            version: 1,
            entities,
            relationships: vec![],
            exposed_properties: vec![],
            metadata: SceneAssetMetadata::default(),
            layers: vec![],
        }
    }

    #[test]
    fn empty_doc_rejected() {
        let doc = make_doc(vec![]);
        let result = export_to_bsn_text(&doc);
        assert!(matches!(result, Err(BsnExportError::EmptyScene)));
    }

    #[test]
    fn non_empty_doc_emits_bsn_block() {
        let entities = vec![SceneAssetEntity {
            local_id: LocalId::new("root"),
            local_path: "root".to_string(),
            name: "Root".to_string(),
            components: vec![crate::document::ComponentInstance {
                type_id: "editor.Name".to_string(),
                values: serde_json::json!({"name": "Root"}),
            }],
        }];
        let doc = make_doc(entities);
        let text = export_to_bsn_text(&doc).unwrap();
        assert!(text.contains("bsn!{"));
        assert!(text.contains("#root"));
        assert!(text.contains("Name(\"Root\")"));
    }

    #[test]
    fn no_legacy_commands_wrapper() {
        // Defense: .bsn files must not contain Rust wrapper syntax
        let entities = vec![SceneAssetEntity {
            local_id: LocalId::new("root"),
            local_path: "root".to_string(),
            name: "Root".to_string(),
            components: vec![],
        }];
        let doc = make_doc(entities);
        let text = export_to_bsn_text(&doc).unwrap();
        assert!(!text.contains("commands.spawn_scene_list"));
        assert!(!text.contains("bsn_list!["));
        assert!(!text.contains(".unwrap();"));
    }

    #[test]
    fn children_blocks_use_bsn_native_commas() {
        // Children blocks should NOT have a trailing comma after each entry
        let root = SceneAssetEntity {
            local_id: LocalId::new("root"),
            local_path: "root".to_string(),
            name: "Root".to_string(),
            components: vec![],
        };
        let child = SceneAssetEntity {
            local_id: LocalId::new("child"),
            local_path: "root/child".to_string(),
            name: "Child".to_string(),
            components: vec![],
        };
        let mut doc = make_doc(vec![root.clone(), child.clone()]);
        doc.relationships.push(crate::scene_asset::SceneAssetRelationship {
            from_local_id: LocalId::new("root"),
            to_local_id: LocalId::new("child"),
            kind: RelationshipKind::Child,
            field_path: None,
        });
        let text = export_to_bsn_text(&doc).unwrap();
        // After Children [ we should see the child block, and after the
        // closing `}` of the child block there should NOT be a `,`.
        assert!(text.contains("Children ["));
        // The text between the child block's `}` and the `]` of Children
        // should be just whitespace and `]`, no comma.
        let child_open_idx = text.find("#child").unwrap();
        let child_close_idx = text[child_open_idx..].find("}").unwrap() + child_open_idx;
        let children_close_idx = text[child_close_idx..].rfind("]").unwrap() + child_close_idx;
        let between = &text[child_close_idx + 1..children_close_idx];
        assert!(!between.contains(','), "between child close and Children close found: {:?}", between);
    }

    #[test]
    fn logic_role_doc_rejected_with_unsupported_shape() {
        // Logic role documents must be rejected before IR build
        let entities = vec![SceneAssetEntity {
            local_id: LocalId::new("root"),
            local_path: "root".to_string(),
            name: "Root".to_string(),
            components: vec![crate::document::ComponentInstance {
                type_id: "editor.Name".to_string(),
                values: serde_json::json!({"name": "Root"}),
            }],
        }];
        let mut doc = make_doc(entities);
        doc.role = crate::scene_asset::SceneAssetRole::Logic;
        let result = export_to_bsn_text(&doc);
        assert!(matches!(
            result,
            Err(BsnExportError::UnsupportedShape(ref s)) if s.contains("logic")
        ));
    }

    #[test]
    fn actor_role_still_exports_successfully() {
        // Regression: Actor role must still work (non_empty_doc_emits_bsn_block)
        let entities = vec![SceneAssetEntity {
            local_id: LocalId::new("root"),
            local_path: "root".to_string(),
            name: "Root".to_string(),
            components: vec![crate::document::ComponentInstance {
                type_id: "editor.Name".to_string(),
                values: serde_json::json!({"name": "Root"}),
            }],
        }];
        let doc = make_doc(entities);
        let text = export_to_bsn_text(&doc).unwrap();
        assert!(text.contains("bsn!{"));
        assert!(text.contains("#root"));
        assert!(text.contains("Name(\"Root\")"));
    }
}
