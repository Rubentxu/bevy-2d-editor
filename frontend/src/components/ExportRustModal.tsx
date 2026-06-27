import { useEffect, useState } from "react";
import { exportToRust } from "../services/code-export";
import { getSceneSnapshot } from "../engine-bridge";

interface Props {
  onClose: () => void;
}

export default function ExportRustModal({ onClose }: Props) {
  const [source, setSource] = useState<string>("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function doExport() {
      try {
        const snap = await getSceneSnapshot();
        const json = JSON.stringify(snap);
        const result = await exportToRust(json);
        setSource(result.source);
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    }
    doExport();
  }, []);

  function handleDownload() {
    const blob = new Blob([source], { type: "text/rust" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "scene.rs";
    a.click();
    URL.revokeObjectURL(url);
  }

  async function handleCopy() {
    await navigator.clipboard.writeText(source);
  }

  return (
    <div className="modal-overlay" data-testid="export-rs-modal">
      <div className="modal-content">
        <h2>Export Rust Code</h2>
        {loading && <p>Generating Rust code...</p>}
        {error && <p className="error">{error}</p>}
        {!loading && !error && (
          <>
            <textarea
              readOnly
              value={source}
              data-testid="export-rs-source"
              className="export-source"
            />
            <div className="modal-actions">
              <button onClick={handleDownload} data-testid="export-rs-download-btn">
                Download scene.rs
              </button>
              <button onClick={handleCopy} data-testid="export-rs-copy-btn">
                Copy to Clipboard
              </button>
              <button onClick={onClose}>Close</button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
