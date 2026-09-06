/**
 * editor-ready contract audit
 *
 * This file documents the audit of readiness polling patterns in the codebase.
 * Per spec §6.1 and implementation-plan B1.2, this audit confirms that
 * all readiness polling has been consolidated to use the single waitForEditorReady
 * helper from utils/waitForEditorReady.ts.
 *
 * Audit results (2026-09-06):
 * - Total call sites using waitForEditorReady: 7
 *   - useAIAssistant.ts: 1 call
 *   - useLogicGraph.ts: 1 call
 *   - importers.ts: 4 calls
 *   - EditorGateway.ts: 1 call (internal)
 * - Total call sites using gateway.whenReady(): 2
 *   - bridge-call.ts: 1 call
 *   - scenes.ts: 1 call
 * - Total setTimeout/setInterval usages: ~15 files
 *   - All are UI-related timers (hover, debounce, data refresh)
 *   - None are readiness polling
 *
 * Conclusion: No setTimeout-based readiness polling exists in the codebase.
 * The readiness contract is properly implemented via waitForEditorReady.
 *
 * The editor-ready module at src/editor-ready/ re-exports waitForEditorReady
 * and provides additional typed surface (READY_STATE enum, EditorReadyEvent,
 * onEditorReady listener) for use by components and tests.
 */
