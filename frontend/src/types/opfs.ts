/**
 * Canonical OpfsResult envelope shape.
 *
 * Used by all WASM bindings exposed via `opfs-bridge.ts` and consumed by
 * service layers (`services/code-files.ts`, etc.). The canonical shape is
 * broad optional fields (`{ok, value?, error?}`) per `opfs-bridge.ts` and
 * `design.md` §Interfaces/Contracts.
 *
 * Services should narrow this discriminated-union-style at their API
 * boundary if they want strict typing (see `services/code-files.ts` for an
 * example), but the on-the-wire JSON shape from WASM is always this.
 */
export interface OpfsResult<T = void> {
  ok: boolean;
  value?: T;
  error?: string;
}
