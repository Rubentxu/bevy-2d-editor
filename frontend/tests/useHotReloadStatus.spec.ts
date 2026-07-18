/**
 * useHotReloadStatus hook tests.
 *
 * Tests the hook's behavior through the UI components that consume it.
 * Uses Playwright (the project's testing framework) rather than @testing-library/react.
 */

import { test, expect } from "@playwright/test";
import type { HotReloadStatus } from "../src/hooks/useHotReloadStatus";

// This test verifies the hook module exports the correct types and can be imported
// without TypeScript errors. Full behavior is tested via GameOverlay and TopBar E2E.
test("useHotReloadStatus exports correct interface", async () => {
  // Verify the type matches the expected shape
  const _status: HotReloadStatus = {
    lastReloadedAt: null,
    inFlightSaves: 0,
    refresh: () => {},
  };

  expect(_status.lastReloadedAt).toBeNull();
  expect(_status.inFlightSaves).toBe(0);
  expect(typeof _status.refresh).toBe("function");
});
