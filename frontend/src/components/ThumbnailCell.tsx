/**
 * ThumbnailCell (ADR-0026)
 *
 * Renders a 64×64 inline preview for a single Asset Browser row. The
 * component owns all preview-related concerns: lazy load, cache, error
 * fallback, cleanup. It is intentionally a no-op on `null`/`undefined`
 * resource paths and on non-image MIME types — a placeholder is the
 * synchronous default so the table is never blocked on I/O.
 *
 * Lifecycle:
 * 1. Mount → render placeholder synchronously.
 * 2. `IntersectionObserver` fires when the row scrolls into view →
 *    `readAssetFileBytes` → `assetThumbnails.getOrInsert` → set Blob URL.
 * 3. Re-render swaps placeholder for `<img>`.
 * 4. Unmount: `IntersectionObserver` disconnects. The Blob URL stays
 *    in the LRU cache; revocation happens on eviction (cache owns the
 *    URL lifecycle, not the component).
 */

import { useEffect, useRef, useState } from "react";
import { readAssetFileBytes } from "../services/asset-files";
import * as assetThumbnails from "../services/asset-thumbnails";

interface ThumbnailCellProps {
  /** Asset catalog row id. Used as a stable React key only. */
  assetId: string;
  /**
   * Resource path under OPFS `resources/` (e.g., "characters/player.png").
   * When null/undefined/empty, the placeholder renders immediately.
   */
  resourcePath?: string | null;
}

const THUMB_SIZE = 64;
const MIME_BY_EXT: Record<string, string> = {
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".gif": "image/gif",
  ".webp": "image/webp",
  ".svg": "image/svg+xml",
};

/**
 * Resolve the MIME type for `path` based on its file extension. Returns
 * `null` if the extension is not a recognised image type — the caller
 * must then render the placeholder (no `readAssetFileBytes` call).
 */
function mimeFor(path: string): string | null {
  const idx = path.lastIndexOf(".");
  if (idx < 0) return null;
  return MIME_BY_EXT[path.slice(idx).toLowerCase()] ?? null;
}

export default function ThumbnailCell({ resourcePath }: ThumbnailCellProps) {
  const [blobUrl, setBlobUrl] = useState<string | null>(null);
  const containerRef = useRef<HTMLSpanElement>(null);
  const loadedRef = useRef(false);

  useEffect(() => {
    // Defensive: skip work for null/empty paths or non-image MIMEs.
    if (!resourcePath || !mimeFor(resourcePath)) return;
    if (loadedRef.current) return;

    const el = containerRef.current;
    const mime = mimeFor(resourcePath)!;

    const loadAndSet = async () => {
      try {
        const bytes = await readAssetFileBytes(resourcePath);
        // Cast: TypeScript 5.7 narrows Uint8Array's buffer to
        // ArrayBufferLike (SharedArrayBuffer | ArrayBuffer), but
        // `new Blob` only accepts ArrayBuffer-backed views. The
        // WASM bridge never returns SharedArrayBuffer.
        const blob = new Blob([bytes as BlobPart], { type: mime });
        const thumb = await assetThumbnails.getOrInsert(
          resourcePath,
          mime,
          async () => blob,
        );
        setBlobUrl(thumb.blobUrl);
      } catch {
        // Swallow: placeholder is the visible default. The cache is
        // not mutated on factory failure.
      }
    };

    // Fallback when IntersectionObserver is unavailable (very old
    // browser): load immediately. The placeholder renders
    // synchronously in either branch.
    if (!el || typeof IntersectionObserver === "undefined") {
      loadedRef.current = true;
      void loadAndSet();
      return;
    }

    const io = new IntersectionObserver((entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting && !loadedRef.current) {
          loadedRef.current = true;
          io.disconnect();
          void loadAndSet();
        }
      }
    });
    io.observe(el);
    return () => io.disconnect();
  }, [resourcePath]);

  // No preview or non-image MIME: render the placeholder synchronously.
  if (!resourcePath || !mimeFor(resourcePath) || !blobUrl) {
    return (
      <span
        ref={containerRef}
        data-testid="thumbnail-placeholder"
        className="thumb-placeholder"
        aria-hidden="true"
      >
        🖼
      </span>
    );
  }

  return (
    <img
      data-testid="thumbnail-img"
      className="thumb-img"
      src={blobUrl}
      width={THUMB_SIZE}
      height={THUMB_SIZE}
      loading="lazy"
      decoding="async"
      alt=""
    />
  );
}
