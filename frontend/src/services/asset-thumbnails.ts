/**
 * Asset thumbnail cache (ADR-0026).
 *
 * Module-level singleton that owns the lifecycle of `URL.createObjectURL`
 * results for Asset Browser thumbnails. Capacity is hard-capped at
 * `MAX_ENTRIES`; on insert when at capacity, the entry with the lowest
 * `lastUsed` is evicted and its Blob URL revoked.
 *
 * Invariant: `cache.size <= MAX_ENTRIES` at all times (see design.md §D4.3).
 */

const MAX_ENTRIES = 32;

const cache = new Map<string, AssetThumbnail>();
let clock = 0;

export interface AssetThumbnail {
  blobUrl: string;
  mime: string;
  /** Monotonically increasing "time" stamp; higher = more recently used. */
  lastUsed: number;
}

/**
 * Returns the cached `AssetThumbnail` for `resourcePath` if present,
 * otherwise calls `factory()` to produce a `Blob`, creates a Blob URL,
 * stores the result, and returns it. On cache-cap overflow, the LRU
 * entry is evicted and its Blob URL revoked.
 *
 * If `factory()` rejects, the error is propagated and the cache is
 * NOT mutated.
 */
export async function getOrInsert(
  resourcePath: string,
  mime: string,
  factory: () => Promise<Blob>,
): Promise<AssetThumbnail> {
  const existing = cache.get(resourcePath);
  if (existing) {
    existing.lastUsed = ++clock;
    return existing;
  }

  if (cache.size >= MAX_ENTRIES) {
    let lruKey: string | null = null;
    let lruTime = Infinity;
    for (const [key, value] of cache) {
      if (value.lastUsed < lruTime) {
        lruTime = value.lastUsed;
        lruKey = key;
      }
    }
    if (lruKey !== null) {
      const evicted = cache.get(lruKey);
      if (evicted) {
        URL.revokeObjectURL(evicted.blobUrl);
      }
      cache.delete(lruKey);
    }
  }

  const blob = await factory();
  const blobUrl = URL.createObjectURL(blob);
  const entry: AssetThumbnail = { blobUrl, mime, lastUsed: ++clock };
  cache.set(resourcePath, entry);
  return entry;
}

/**
 * Drops the entry for `resourcePath` from the cache and revokes its
 * Blob URL. No-op if the path is not present.
 */
export function revoke(resourcePath: string): void {
  const entry = cache.get(resourcePath);
  if (!entry) return;
  URL.revokeObjectURL(entry.blobUrl);
  cache.delete(resourcePath);
}

/**
 * Drops all entries from the cache and revokes every Blob URL. Used
 * by tests and the hot-reload path when an asset file is deleted.
 */
export function clear(): void {
  for (const entry of cache.values()) {
    URL.revokeObjectURL(entry.blobUrl);
  }
  cache.clear();
}

/**
 * Returns the current entry count. Test-only seam.
 */
export function size(): number {
  return cache.size;
}

/**
 * Returns the hard cap on the number of entries. Test-only seam.
 */
export function capacity(): number {
  return MAX_ENTRIES;
}
