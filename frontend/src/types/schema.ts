/**
 * ComponentSchema — Hito 4 Order 7 (scene-component-authoring) extension.
 *
 * Mirrors the Rust `ComponentSchema` in `crates/editor-core/src/schema.rs`.
 * Backward-compat: v0.72.0 and earlier schemas serialized without the 3
 * new fields. The Rust side uses `#[serde(default)]` so the new fields are
 * always optional here.
 */

export type SchemaKind = "simple" | "scene_component";

/** Mirrors Rust `FieldType` enum (untagged for backward compat). */
export type FieldType =
  | "String"
  | "F32"
  | "Bool"
  | "Vec2"
  | "Color"
  | "Anchor"
  | "AssetReference"
  | string  // ComponentRef (serialized as plain string) or Enum (object)
  | { Enum: { variants: string[] } };

export interface Constraint {
  NonEmpty?: null;
  Min?: number;
  Max?: number;
}

export interface FieldDef {
  name: string;
  field_type: FieldType;
  default: unknown;
  constraints?: Constraint[];
}

export interface SourceLocation {
  file_id: string;
  line: number;
  column: number;
}

export interface ComponentSchema {
  type_id: string;
  display_name: string;
  fields: FieldDef[];
  exports_to_bevy: boolean;
  version?: string;
  source_location?: SourceLocation | null;
  /** Hito 4 Order 7: schema discriminator. Default "simple". */
  kind?: SchemaKind;
  /** Hito 4 Order 7: when kind = "scene_component", references a SceneAsset. */
  bound_scene_asset_ref?: string | null;
  /** Hito 4 Order 7: auto-spawn bound scene on instance. Default true. */
  auto_spawn?: boolean;
}

/**
 * Helper: is this schema a SceneComponent?
 */
export function isSceneComponent(schema: ComponentSchema): boolean {
  return schema.kind === "scene_component";
}
