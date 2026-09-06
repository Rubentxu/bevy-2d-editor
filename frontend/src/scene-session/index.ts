/**
 * Scene Session module.
 *
 * Per spec §6.4 (scene-session-encapsulation), this module encapsulates:
 * - SceneDocument state
 * - OperationLog (for undo/redo)
 * - Dirty tracking
 * - Scene-switch invariants
 *
 * Module-private state (not exported):
 * - document: the active SceneDocument
 * - operationLog: the undo/redo log
 * - dirty: whether the scene has unsaved changes
 * - activeSceneId: the current scene identifier
 *
 * Public API:
 * - dispatchSceneCommand(cmd, envelope): dispatch a command and update state
 * - getActiveSceneSnapshot(): get current scene snapshot
 * - markClean(): mark the scene as saved (dirty = false)
 * - requestSceneSwitch(target): request a scene switch (may trigger dirty guard)
 *
 * This module extends the existing useSceneState hook with module-level
 * encapsulation that prevents direct mutation of internal state.
 *
 * IMPORTANT: per ADR-0007, Scene, Scene Asset, and LogicGraph operation
 * logs are SEPARATE. This module only manages the Scene operation log.
 */

import type { SceneDocument, DispatchResult } from "../hooks/useSceneState";
import { getEditorGateway } from "../services/EditorGateway";

// ============================================================================
// Module-private state
// ============================================================================

let _document: SceneDocument | null = null;
let _dirty: boolean = false;
let _activeSceneId: string | null = null;
let _operationLog: object[] = [];

// ============================================================================
// Internal helpers
// ============================================================================

async function _refreshFromGateway(): Promise<SceneDocument | null> {
  const gateway = getEditorGateway();
  const result = await gateway.getSceneSnapshot();
  if (result.ok) {
    return result.value as SceneDocument;
  }
  console.error("scene-session: refresh failed:", result.error);
  return _document;
}

// ============================================================================
// Public API
// ============================================================================

/**
 * Dispatch a scene command through the gateway.
 * Updates module-private state after successful dispatch.
 */
export async function dispatchSceneCommand(
  envelope: object,
): Promise<DispatchResult> {
  const gateway = getEditorGateway();
  try {
    const parsed = await gateway.dispatchCommand(envelope);
    if (parsed.snapshot) {
      _document = parsed.snapshot as SceneDocument;
      // Record in operation log (for undo/redo)
      if (parsed.inverse) {
        _operationLog.push(parsed.inverse);
      }
      _dirty = true;
    }
    return {
      inverse: parsed.inverse,
      snapshot: parsed.snapshot as SceneDocument | undefined,
      error: parsed.error,
    };
  } catch (e) {
    const msg = String(e);
    console.error("scene-session: dispatch failed:", e);
    return { error: msg };
  }
}

/**
 * Get the current scene snapshot.
 */
export async function getActiveSceneSnapshot(): Promise<SceneDocument | null> {
  return _refreshFromGateway();
}

/**
 * Get the current scene snapshot synchronously (may be stale).
 */
export function getActiveSceneSnapshotSync(): SceneDocument | null {
  return _document;
}

/**
 * Mark the scene as clean (no unsaved changes).
 */
export function markClean(): void {
  _dirty = false;
}

/**
 * Check if the current scene has unsaved changes.
 */
export function isDirty(): boolean {
  return _dirty;
}

/**
 * Request a switch to a different scene.
 * Returns true if the switch is allowed (clean or force=true).
 * Returns false if the switch should be blocked by a dirty guard.
 */
export async function requestSceneSwitch(
  targetSceneId: string,
  force: boolean = false,
): Promise<{ allowed: boolean; reason?: string }> {
  if (_dirty && !force) {
    return {
      allowed: false,
      reason: "Unsaved changes. Save or discard before switching.",
    };
  }
  // Perform the switch
  _activeSceneId = targetSceneId;
  // Refresh state from gateway
  await _refreshFromGateway();
  return { allowed: true };
}

/**
 * Get the current active scene ID.
 */
export function getActiveSceneId(): string | null {
  return _activeSceneId;
}

/**
 * Get the operation log length (for diagnostics).
 */
export function getOperationLogSize(): number {
  return _operationLog.length;
}

// ============================================================================
// Type exports (for consumers)
// ============================================================================

export type { SceneDocument, DispatchResult };
