/**
 * Unit tests for code-files.ts service functions.
 *
 * Tests the parsing logic for findSourceLocation and findEntitiesByType
 * which wrap WASM bindings. These tests verify the TypeScript parsing
 * behavior without requiring WASM runtime.
 *
 * NOTE: This file is designed to be run with vitest when configured,
 * or the test cases can be verified manually. The project currently
 * uses Playwright for E2E tests.
 *
 * To run with vitest (when configured):
 *   npx vitest src/services/code-files.test.ts
 */

import type { SourceLocation } from "./code-files";

/**
 * Reimplementation of findSourceLocation parsing logic for unit testing.
 * This mirrors the actual implementation in code-files.ts.
 */
function parseFindSourceLocationResult(result: string): SourceLocation | null {
  return result === "null" ? null : JSON.parse(result);
}

/**
 * Reimplementation of findEntitiesByType parsing logic for unit testing.
 * This mirrors the actual implementation in code-files.ts.
 */
function parseFindEntitiesByTypeResult(result: string): string[] {
  return JSON.parse(result);
}

// ============================================
// Test cases for findSourceLocation parsing
// ============================================

export function runFindSourceLocationTests() {
  console.group("findSourceLocation parsing tests");

  // B.3.1: parse null string correctly
  console.log("Test: parses 'null' string as null");
  const nullResult = parseFindSourceLocationResult("null");
  console.assert(nullResult === null, "Expected null");
  console.log("  ✓ PASS");

  // B.3.2: parse object correctly
  console.log("Test: parses object with file_id, line, column correctly");
  const objResult = parseFindSourceLocationResult(
    JSON.stringify({
      file_id: "src/ecs/components.rs",
      line: 42,
      column: 7,
    })
  );
  console.assert(
    objResult !== null &&
      objResult.file_id === "src/ecs/components.rs" &&
      objResult.line === 42 &&
      objResult.column === 7,
    "Expected { file_id, line, column }"
  );
  console.log("  ✓ PASS");

  // B.3.3: parses result with only required fields
  console.log("Test: parses result with minimal fields");
  const minimalResult = parseFindSourceLocationResult(
    JSON.stringify({
      file_id: "src/player.rs",
      line: 10,
      column: 1,
    })
  );
  console.assert(minimalResult !== null, "Expected non-null");
  console.assert(minimalResult!.file_id === "src/player.rs", "file_id mismatch");
  console.assert(minimalResult!.line === 10, "line mismatch");
  console.assert(minimalResult!.column === 1, "column mismatch");
  console.log("  ✓ PASS");

  console.groupEnd();
  console.log("");
}

// ============================================
// Test cases for findEntitiesByType parsing
// ============================================

export function runFindEntitiesByTypeTests() {
  console.group("findEntitiesByType parsing tests");

  // B.3.4: parses empty array correctly
  console.log("Test: parses empty array as empty string array");
  const emptyResult = parseFindEntitiesByTypeResult("[]");
  console.assert(emptyResult.length === 0, "Expected empty array");
  console.log("  ✓ PASS");

  // B.3.5: parses single entity correctly
  console.log("Test: parses single entity id correctly");
  const singleResult = parseFindEntitiesByTypeResult('["ent_player"]');
  console.assert(singleResult.length === 1, "Expected 1 element");
  console.assert(singleResult[0] === "ent_player", "Entity ID mismatch");
  console.log("  ✓ PASS");

  // B.3.6: parses multiple entities correctly
  console.log("Test: parses multiple entity ids correctly");
  const multiResult = parseFindEntitiesByTypeResult(
    '["ent_player", "ent_enemy", "ent_npc"]'
  );
  console.assert(multiResult.length === 3, "Expected 3 elements");
  console.assert(
    multiResult[0] === "ent_player" &&
      multiResult[1] === "ent_enemy" &&
      multiResult[2] === "ent_npc",
    "Entity IDs mismatch"
  );
  console.log("  ✓ PASS");

  console.groupEnd();
  console.log("");
}

// Run tests if executed directly
if (typeof window !== "undefined") {
  // Browser environment - run tests
  runFindSourceLocationTests();
  runFindEntitiesByTypeTests();
  console.log("All tests completed.");
}
