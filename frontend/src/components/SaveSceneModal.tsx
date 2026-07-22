import { useEffect, useRef, useState } from "react";

interface Props {
  defaultName?: string;
  onSave: (name: string) => void;
  onCancel: () => void;
}

/**
 * Modal dialog for naming a scene before saving (replaces window.prompt).
 * Enter submits, Escape cancels. Input is focused on mount.
 */
export default function SaveSceneModal({
  defaultName = "level_01",
  onSave,
  onCancel,
}: Props) {
  const [name, setName] = useState(defaultName);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
    inputRef.current?.select();
  }, []);

  const handleSubmit = (e?: React.FormEvent) => {
    e?.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) return;
    onSave(trimmed);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.stopPropagation();
      onCancel();
    }
  };

  return (
    <div
      className="dialog-overlay"
      data-testid="save-scene-modal"
      onClick={(e) => {
        if (e.target === e.currentTarget) onCancel();
      }}
    >
      <div className="dialog" role="dialog" aria-modal="true">
        <h3>Save Scene</h3>
        <form onSubmit={handleSubmit}>
          <label
            htmlFor="save-scene-name"
            style={{
              display: "block",
              marginBottom: 6,
              fontSize: 12,
              color: "#a0aec0",
            }}
          >
            Scene name
          </label>
          <input
            id="save-scene-name"
            ref={inputRef}
            type="text"
            className="save-scene-input"
            data-testid="save-scene-name-input"
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={handleKeyDown}
            style={{
              width: "100%",
              padding: "8px 10px",
              borderRadius: 4,
              border: "1px solid #4a5568",
              background: "#1a2744",
              color: "#e2e8f0",
              fontSize: 14,
              marginBottom: 16,
            }}
          />
          <div className="dialog-actions">
            <button
              type="button"
              onClick={onCancel}
              data-testid="save-scene-cancel-btn"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="primary"
              data-testid="save-scene-confirm-btn"
              disabled={!name.trim()}
            >
              Save
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
