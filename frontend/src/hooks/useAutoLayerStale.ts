import { useState, useEffect } from 'react';
import { isAutoLayerStale } from '../services/autoLayer';

/**
 * React hook that tracks whether an AutoLayer's cached tile grid is stale.
 *
 * An AutoLayer cache is stale when the source TileLayer has been modified
 * (paint/erase) since the cache was last built via `regenerateAutoLayer`.
 *
 * @param assetRef - Logical path of the scene asset (e.g. "levels/world1")
 * @param layerId  - The AutoLayer's stable id string
 * @returns `true` if the cached grid needs regeneration, `false` if up-to-date
 */
export function useAutoLayerStale(assetRef: string, layerId: string): boolean {
  const [stale, setStale] = useState(false);

  useEffect(() => {
    let cancelled = false;
    isAutoLayerStale(assetRef, layerId)
      .then((s) => { if (!cancelled) setStale(s); })
      .catch(() => {});
    return () => { cancelled = true; };
  }, [assetRef, layerId]);

  return stale;
}
