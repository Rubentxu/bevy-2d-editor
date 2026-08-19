/**
 * Hot-reload bus integration tests.
 *
 * Tests the typed event bus service for hot-reload notifications.
 * The bus decouples save hooks from WASM invalidation calls.
 */

import { test, expect } from "@playwright/test";
import type { HotReloadSourceEvent, HotReloadAssetEvent } from "../src/services/hot-reload";

// These functions don't exist yet — RED phase
// @ts-expect-error — module not yet implemented
import { subscribe, emit, inFlightSaveCounter } from "../src/services/hot-reload";

test.describe("hot-reload bus", { tag: ["@full"] }, () => {
  test("subscribe receives source event after emit", { tag: ["@full"] }, async () => {
    const received: HotReloadSourceEvent[] = [];
    const unsub = subscribe("hot-reload-source", (event) => {
      received.push(event as HotReloadSourceEvent);
    });

    emit({ type: "hot-reload-source", fileId: "src/main.rs" });

    expect(received).toHaveLength(1);
    expect(received[0].fileId).toBe("src/main.rs");

    unsub();
  });

  test("subscribe receives asset event after emit", { tag: ["@full"] }, async () => {
    const received: HotReloadAssetEvent[] = [];
    const unsub = subscribe("hot-reload-asset", (event) => {
      received.push(event as HotReloadAssetEvent);
    });

    emit({ type: "hot-reload-asset", assetId: "player.bsn" });

    expect(received).toHaveLength(1);
    expect(received[0].assetId).toBe("player.bsn");

    unsub();
  });

  test("unsubscribe stops delivery", { tag: ["@full"] }, async () => {
    const received: unknown[] = [];
    const unsub = subscribe("hot-reload-source", (event) => {
      received.push(event);
    });

    unsub();
    emit({ type: "hot-reload-source", fileId: "x.rs" });

    expect(received).toHaveLength(0);
  });

  test("subscriber error does not throw", { tag: ["@full"] }, async () => {
    const handler = () => {
      throw new Error("handler error");
    };
    subscribe("hot-reload-source", handler);

    // Should not throw — errors are caught and logged
    expect(() => emit({ type: "hot-reload-source", fileId: "x.rs" })).not.toThrow();
  });

  test("inFlightSaveCounter increments and decrements", { tag: ["@full"] }, async () => {
    expect(inFlightSaveCounter.value).toBe(0);

    inFlightSaveCounter.incr();
    expect(inFlightSaveCounter.value).toBe(1);

    inFlightSaveCounter.incr();
    expect(inFlightSaveCounter.value).toBe(2);

    inFlightSaveCounter.decr();
    expect(inFlightSaveCounter.value).toBe(1);

    inFlightSaveCounter.decr();
    expect(inFlightSaveCounter.value).toBe(0);
  });
});
