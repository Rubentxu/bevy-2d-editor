/**
 * perfBudget.ts — Wall-clock performance budget helper for the G6
 * performance corpus specs (frontend/tests/perf-*.spec.ts).
 *
 * Each perf spec declares a `PerfBudgetSpec { name, softMs, hardMs }`
 * at the top of its file, then wraps the operation under measurement
 * in `assertWithinBudget(spec, fn)`. The function returns the measured
 * time in ms (for capture in the verify report) and:
 *
 *   - throws if the wall time exceeds `hardMs` (fails the spec);
 *   - logs a `console.warn` if the wall time exceeds `softMs` (informational);
 *   - returns silently when the wall time is within both budgets.
 *
 * Per the G6 specification (`docs/sddk/g6-performance-corpus/specification.md`
 * §5), the soft budget is a warning, not a failure: the budget philosophy
 * is "fail loudly when regression crosses the hard limit, log a warning
 * before it crosses the warn limit so we can tighten the spec later".
 */

export interface PerfBudgetSpec {
  /** Display name used in assertion messages and warnings. */
  readonly name: string;
  /** Soft budget in ms — exceeded logs a warning but does not fail. */
  readonly softMs: number;
  /** Hard budget in ms — exceeded fails the test. */
  readonly hardMs: number;
}

/**
 * Measure the wall-clock time of an async function in milliseconds.
 *
 * The function uses `Date.now()` (millisecond resolution) instead of
 * `performance.now()` because `Date.now()` is what Playwright traces
 * surface and what the CI logs show; matching the trace format makes
 * the verify report easier to compare with CI artefacts.
 */
export async function measureMs(fn: () => Promise<void>): Promise<number> {
  const start = Date.now();
  await fn();
  return Date.now() - start;
}

/**
 * Run `fn` and assert the wall-clock time is within the hard budget.
 *
 * Behaviour:
 *
 *   - Returns the measured time in ms for capture.
 *   - Throws `Error` if `ms > hardMs` — this fails the Playwright spec.
 *   - Logs `console.warn(...)` if `ms > softMs` — does not fail.
 *
 * @param spec  Budget declaration with name + soft + hard thresholds.
 * @param fn    Async operation to time.
 */
export async function assertWithinBudget(
  spec: PerfBudgetSpec,
  fn: () => Promise<void>,
): Promise<number> {
  const ms = await measureMs(fn);
  if (ms > spec.hardMs) {
    throw new Error(
      `perf budget FAIL: ${spec.name} took ${ms} ms (hard budget ${spec.hardMs} ms)`,
    );
  }
  if (ms > spec.softMs) {
    // Soft budget exceeded — log a warning but do not fail. The verify
    // phase will collect these warnings into a "soft violations" list.
    // eslint-disable-next-line no-console
    console.warn(
      `perf budget WARN: ${spec.name} took ${ms} ms (soft ${spec.softMs} ms; hard ${spec.hardMs} ms)`,
    );
  }
  return ms;
}
