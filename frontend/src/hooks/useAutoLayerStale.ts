import { useState, useEffect, useCallback } from 'react';
import { isAutoLayerStale } from '../services/autoLayer';

/**
 * React hook that tracks whether an AutoLayer's cached tile grid is stale,
 * and provides a manual override for the regenerate flow.
 *
 * An AutoLayer cache is stale when the source TileLayer has been modified
 * (paint/erase) since the cache was last built via `regenerateAutoLayer`.
 *
 * @param assetRef - Logical path of the scene asset (e.g. "levels/world1")
 * @param layerId  - The AutoLayer's stable id string
 * @returns [stale, setStale, refreshStale] tuple
 */
export function useAutoLayerStale(assetRef: string, layerId: string): [boolean, (v: boolean) => void, () => void] {
  const [stale, setStale] = useState(false);

  const refreshStale = useCallback(() => {
    isAutoLayerStale(assetRef, layerId)
      .then((s) => setStale(s))
      .catch(() => setStale(false));
  }, [assetRef, layerId]);

  useEffect(() => {
    let cancelled = false;
    isAutoLayerStale(assetRef, layerId)
      .then((s) => { if (!cancelled) setStale(s); })
      .catch(() => {});
    return () => { cancelled = true; };
  }, [assetRef, layerId]);

  return [stale, setStale, refreshStale];
}
