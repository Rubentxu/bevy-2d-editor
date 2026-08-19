/**
 * Shared bridge-access helper for service adapters (Wave D1).
 *
 * Services keep their public signatures but route every WASM call through
 * the typed EditorGateway bridge — `window.*` is never touched outside
 * engine-bridge/EditorGateway.
 */
import { getEditorGateway, type WindowWithBridge } from "./EditorGateway";

export function bridge(): WindowWithBridge | null {
  return getEditorGateway().bridge;
}

export async function bridgeReady(): Promise<void> {
  await getEditorGateway().whenReady();
}

/**
 * Call a bridge binding by name with the given args. Throws if the binding
 * is missing (fail-fast contract, same as the old window.* access).
 */
export async function callBridge<T = unknown>(
  name: keyof WindowWithBridge,
  ...args: unknown[]
): Promise<T> {
  const b = bridge();
  const fn = b?.[name];
  if (typeof fn !== "function") {
    throw new Error(`${String(name)} export not available`);
  }
  const result = await (fn as (...a: unknown[]) => unknown)(...args);
  return result as T;
}

/**
 * Synchronous bridge call for components that read values without await
 * (e.g. useMemo/useEffect bodies). Mirrors the legacy `window.*` sync
 * access; throws if the binding is missing.
 */
export function callBridgeSync<T = unknown>(
  name: keyof WindowWithBridge,
  ...args: unknown[]
): T {
  const b = bridge();
  const fn = b?.[name];
  if (typeof fn !== "function") {
    throw new Error(`${String(name)} export not available`);
  }
  return (fn as (...a: unknown[]) => unknown)(...args) as T;
}
