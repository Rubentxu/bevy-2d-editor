/**
 * useDockPrefs migration tests (v3 → v4).
 *
 * Run with: npx ts-node --esm src/hooks/useDockPrefs.migration.test.ts
 * Or: npx tsx src/hooks/useDockPrefs.migration.test.ts
 *
 * Also verified via Playwright in:
 *   tests/ux-dock.spec.ts (migration smoke test)
 */

import { migratePrefs, DEFAULT_DOCK_PREFS, SCHEMA_VERSION } from "./useDockPrefs";

// ---------------------------------------------------------------------------
// Test: v3 fixture migrates to v4 with change-workbench in bottom region
// ---------------------------------------------------------------------------

const V3_FIXTURE = {
  schemaVersion: 3,
  statusBar: { height: 24 },
  activePreset: "default",
  left: { width: 280, visible: true },
  right: {
    width: 320,
    visible: true,
    outlineVisible: true,
    outlineCollapsed: false,
    propertiesVisible: true,
    propertiesCollapsed: false,
    topHeight: 60,
  },
  bottom: { height: 240, visible: true },
  panelRegions: {
    assets: "left",
    outline: "right",
    properties: "right",
    bottom: "bottom",
    // "change-workbench" is NEW in v4 — absent from v3
  },
  floats: {},
  presets: {},
};

function assert(condition: boolean, message: string) {
  if (!condition) throw new Error(`Assertion failed: ${message}`);
}

export function runMigrationTests() {
  const result = migratePrefs(V3_FIXTURE);

  // schemaVersion should be bumped to 4
  assert(
    result.schemaVersion === 4,
    `Expected schemaVersion 4, got ${result.schemaVersion}`,
  );

  // "change-workbench" should be present in panelRegions
  assert(
    "change-workbench" in result.panelRegions,
    "change-workbench key must be present in panelRegions after v3→v4 migration",
  );

  // "change-workbench" should default to "bottom"
  assert(
    result.panelRegions["change-workbench"] === "bottom",
    `Expected change-workbench → bottom, got ${result.panelRegions["change-workbench"]}`,
  );

  // Other panel regions should be preserved
  assert(
    result.panelRegions.assets === "left",
    `Expected assets → left, got ${result.panelRegions.assets}`,
  );
  assert(
    result.panelRegions.outline === "right",
    `Expected outline → right, got ${result.panelRegions.outline}`,
  );
  assert(
    result.panelRegions.properties === "right",
    `Expected properties → right, got ${result.panelRegions.properties}`,
  );
  assert(
    result.panelRegions.bottom === "bottom",
    `Expected bottom → bottom, got ${result.panelRegions.bottom}`,
  );

  // SCHEMA_VERSION constant should be 4
  assert(
    SCHEMA_VERSION === 4,
    `SCHEMA_VERSION constant should be 4, got ${SCHEMA_VERSION}`,
  );

  // DEFAULT_DOCK_PREFS should reflect v4
  assert(
    DEFAULT_DOCK_PREFS.schemaVersion === 4,
    `DEFAULT_DOCK_PREFS.schemaVersion should be 4, got ${DEFAULT_DOCK_PREFS.schemaVersion}`,
  );
  assert(
    DEFAULT_DOCK_PREFS.panelRegions["change-workbench"] === "bottom",
    `DEFAULT_DOCK_PREFS change-workbench should be bottom`,
  );

  console.log("✅ All v3→v4 migration tests passed");
}

// Self-run
runMigrationTests();
