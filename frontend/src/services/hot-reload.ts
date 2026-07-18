/**
 * Hot-reload event bus — decouples save hooks from WASM invalidation.
 *
 * Architecture:
 *   Save hooks (code-files.ts, asset-files.ts) call emit() on save success.
 *   engine-bridge.ts subscribes and calls window.hot_reload_*_wasm() to
 *   push requests onto the Rust HOT_RELOAD_BUS for next-frame processing.
 *
 * The bus itself is pure and holds no state — subscribers own their side effects.
 */

export interface HotReloadSourceEvent {
  type: "hot-reload-source";
  fileId: string;
}

export interface HotReloadAssetEvent {
  type: "hot-reload-asset";
  assetId: string;
}

export type HotReloadEvent = HotReloadSourceEvent | HotReloadAssetEvent;

type Handler = (event: HotReloadEvent) => void;

const handlers = new Map<string, Set<Handler>>();

/**
 * Subscribe to a hot-reload event type.
 * @param type - "hot-reload-source" | "hot-reload-asset"
 * @param handler - Called with the event on emit
 * @returns Unsubscribe function — call to remove this subscription
 */
export function subscribe(type: string, handler: Handler): () => void {
  if (!handlers.has(type)) {
    handlers.set(type, new Set());
  }
  handlers.get(type)!.add(handler);

  return () => {
    handlers.get(type)?.delete(handler);
  };
}

/**
 * Emit a hot-reload event to all subscribers of that type.
 * Subscriber errors are caught and logged — never re-thrown.
 */
export function emit(event: HotReloadEvent): void {
  const set = handlers.get(event.type);
  if (!set) return;

  for (const handler of set) {
    try {
      handler(event);
    } catch (err) {
      console.error(`[hot-reload] handler error for ${event.type}:`, err);
    }
  }
}

/**
 * Tracks in-flight (pending) save operations.
 * Incremented before a save starts, decremented in finally after completion.
 * Used by useHotReloadStatus to detect when all saves have settled.
 */
export const inFlightSaveCounter = {
  value: 0,

  incr() {
    this.value++;
  },

  decr() {
    this.value--;
  },
};
