import { callBridge, bridgeReady } from "./bridge-call";
/**
 * AI-Assisted Editing Frontend Service.
 *
 * Bridges the browser-based editor to the Rust axum proxy that routes to
 * Ollama/OpenAI for AI-suggested scene edit commands.
 *
 * The proxy endpoint is POST /v1/propose and returns CommandEnvelope[] JSON.
 */

// ─────────────────────────────────────────────────────────────────────────────
// Types — mirror of Rust command.rs / ai-proxy types
// ─────────────────────────────────────────────────────────────────────────────

/** Mirrors Rust CommandMetadata.authorship */
export type Authorship = string;

/** Unix milliseconds timestamp */
export type TimestampMs = number;

/** Field path for SetComponentField, e.g. "translation.x" */
export type FieldPath = string;

/** Stable entity ID — opaque immutable identifier */
export type StableId = string;

/** Serialized JSON value (number | boolean | string | object | array | null) */
export type JsonValue = unknown;

/** Component instance values — map of field name to JSON value */
export type ComponentValues = Record<string, JsonValue>;

// ─── Command variants ────────────────────────────────────────────────────────

export type Command =
  | {
      type: "CreateEntity";
      id: StableId;
      name: string;
      components?: ComponentInstance[];
    }
  | { type: "DeleteEntity"; id: StableId }
  | {
      type: "AddComponent";
      entity_id: StableId;
      type_id: string;
      values?: JsonValue;
    }
  | { type: "RemoveComponent"; entity_id: StableId; type_id: string }
  | {
      type: "SetComponentField";
      entity_id: StableId;
      type_id: string;
      field_path: FieldPath;
      value: JsonValue;
    }
  // v0.82 P2 (ADR-0025): mirror of Rust
  // `Command::SetComponentFieldOnMultiple` — applies the same field on
  // the same component to many entities at once. The frontend
  // dispatches one envelope; the Rust processor fans it out into a
  // Batch of per-entity `SetComponentField`s so partial failures roll
  // back atomically (ADR-0025 §D5).
  | {
      type: "SetComponentFieldOnMultiple";
      entity_ids: StableId[];
      type_id: string;
      field_path: FieldPath;
      value: JsonValue;
    }
  | {
      type: "ReparentEntity";
      entity_id: StableId;
      old_parent?: StableId | null;
      new_parent?: StableId | null;
    }
  | {
      type: "RenameEntity";
      entity_id: StableId;
      old_name?: string | null;
      new_name: string;
    }
  | {
      type: "Batch";
      label: string;
      commands: Command[];
    };

/** Mirrors Rust ComponentInstance */
export interface ComponentInstance {
  type_id: string;
  values: ComponentValues;
}

/** Mirrors Rust CommandMetadata */
export interface CommandMetadata {
  authorship: Authorship;
  timestamp: TimestampMs;
  rationale?: string;
  /** Optional model identifier returned by the AI proxy */
  model?: string;
}

/** Mirrors Rust CommandEnvelope */
export interface CommandEnvelope {
  command: Command;
  metadata: CommandMetadata;
}

// ─── Proxy request / response ────────────────────────────────────────────────

/** Request body sent to POST /v1/proxy */
export interface ProposeRequest {
  prompt: string;
  scene_snapshot: unknown;
  schemas: unknown[];
  // Hito 4 Order 6: multi-source context (code-aware-ai). All optional,
  // defaulting to empty when omitted (proxy treats them as #[serde(default)]).
  source_files?: SourceFileRef[];
  logic_graphs?: LogicGraphRef[];
  scene_assets?: SceneAssetContext;
  selected_entity?: SelectedEntity | null;
}

// Re-export types from `types/ai.ts` so existing consumers don't break.
import type {
  SourceFileRef,
  LogicGraphRef,
  SceneAssetContext,
  SelectedEntity,
} from "../types/ai";

/** Response from POST /v1/propose */
export interface ProposeResponse {
  commands: CommandEnvelope[];
}

// ─── Error types ─────────────────────────────────────────────────────────────

export class InvalidRequestError extends Error {
  constructor(message = "Invalid request") {
    super(message);
    this.name = "InvalidRequestError";
  }
}

export class MissingApiKeyError extends Error {
  constructor(message = "API key not configured") {
    super(message);
    this.name = "MissingApiKeyError";
  }
}

export class UpstreamError extends Error {
  constructor(message = "Upstream AI service error") {
    super(message);
    this.name = "UpstreamError";
  }
}

export class NetworkError extends Error {
  constructor(message = "Network error") {
    super(message);
    this.name = "NetworkError";
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// AI Service
// ─────────────────────────────────────────────────────────────────────────────

/** Default timeout for AI proxy requests (30 seconds) */
const DEFAULT_TIMEOUT_MS = 30_000;

/**
 * Send a propose request to the AI-assisted editing proxy.
 *
 * Hito 4 Order 6 (`code-aware-ai`): now accepts an optional `MultiSourceContext`
 * that includes source files, logic graphs, scene assets, and selected entity.
 * Pre-Order-6 callers that pass only `sceneSnapshot` + `schemas` continue to
 * work (the new fields are optional in the request body).
 *
 * @param prompt         Natural-language instruction from the user
 * @param sceneSnapshot  Current SceneDocument snapshot (as returned by get_scene_snapshot)
 * @param schemas        Combined component schemas (as returned by get_combined_schemas_json, parsed)
 * @param proxyUrl       Full URL of the proxy (e.g. http://localhost:11435).
 *                       Pass undefined to use the OPFS-stored setting.
 * @param timeoutMs      Request timeout in ms (default 30_000)
 * @param extraContext   Optional multi-source context (Hito 4 Order 6)
 */
export async function fetchPropose(
  prompt: string,
  sceneSnapshot: unknown,
  schemas: unknown[],
  proxyUrl?: string,
  timeoutMs = DEFAULT_TIMEOUT_MS,
  extraContext?: {
    source_files?: SourceFileRef[];
    logic_graphs?: LogicGraphRef[];
    scene_assets?: SceneAssetContext;
    selected_entity?: SelectedEntity | null;
  },
): Promise<ProposeResponse> {
  const baseUrl =
    proxyUrl ??
    (typeof window !== "undefined" && (window as any).__aiProxyUrlOverride) ??
    (await import("./ai-settings").then((m) => m.getProxyUrl()));
  const url = `${baseUrl}/v1/propose`;

  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);

  try {
    const body: ProposeRequest = {
      prompt,
      scene_snapshot: sceneSnapshot,
      schemas,
      source_files: extraContext?.source_files ?? [],
      logic_graphs: extraContext?.logic_graphs ?? [],
      scene_assets: extraContext?.scene_assets ?? {
        catalog: [],
        selected_body: null,
      },
      selected_entity: extraContext?.selected_entity ?? null,
    };
    const response = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
      signal: controller.signal,
    });

    clearTimeout(timer);

    if (response.status === 400) {
      throw new InvalidRequestError(await response.text());
    }
    if (response.status === 503) {
      throw new MissingApiKeyError(
        "API key missing or invalid — check AI settings",
      );
    }
    if (response.status === 502) {
      throw new UpstreamError("AI service returned an error");
    }
    if (!response.ok) {
      throw new UpstreamError(
        `Proxy returned ${response.status}: ${await response.text()}`,
      );
    }

    const data = (await response.json()) as ProposeResponse;
    return data;
  } catch (err) {
    clearTimeout(timer);
    if (err instanceof InvalidRequestError) throw err;
    if (err instanceof MissingApiKeyError) throw err;
    if (err instanceof UpstreamError) throw err;
    if (err instanceof Error && err.name === "AbortError") {
      throw new NetworkError("Request timed out");
    }
    // Network failure (CORS, DNS, etc.)
    throw new NetworkError(err instanceof Error ? err.message : String(err));
  }
}
