import { useEffect, useRef, useState } from "react";

interface Props {
  title: string;
  label: string;
  placeholder?: string;
  defaultValue?: string;
  onConfirm: (value: string) => void;
  onCancel: () => void;
  validator?: (value: string) => string | null;
}

/**
 * In-app replacement for window.prompt.
 * Enter submits, Escape cancels. Input auto-focuses on mount.
 * Shows inline validation error when validator returns a message.
 */
export default function PromptDialog({
  title,
  label,
  placeholder = "",
  defaultValue = "",
  onConfirm,
  onCancel,
  validator,
}: Props) {
  const [value, setValue] = useState(defaultValue);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
    inputRef.current?.select();
  }, []);

  const handleSubmit = (e?: React.FormEvent) => {
    e?.preventDefault();
    const trimmed = value.trim();
    if (!trimmed) {
      setError("Name cannot be empty.");
      return;
    }
    if (validator) {
      const msg = validator(trimmed);
      if (msg) {
        setError(msg);
        return;
      }
    }
    onConfirm(trimmed);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.stopPropagation();
      onCancel();
    }
  };

  const handleChange = (v: string) => {
    setValue(v);
    if (error) setError(null);
  };

  return (
    <div
      className="dialog-overlay"
      data-testid="prompt-dialog"
      onClick={(e) => {
        if (e.target === e.currentTarget) onCancel();
      }}
    >
      <div className="dialog" role="dialog" aria-modal="true">
        <h3>{title}</h3>
        <form onSubmit={handleSubmit}>
          <label
            htmlFor="prompt-dialog-input"
            style={{
              display: "block",
              marginBottom: 6,
              fontSize: 12,
              color: "#a0aec0",
            }}
          >
            {label}
          </label>
          <input
            id="prompt-dialog-input"
            ref={inputRef}
            type="text"
            className="prompt-dialog-input"
            data-testid="prompt-dialog-input"
            placeholder={placeholder}
            value={value}
            onChange={(e) => handleChange(e.target.value)}
            onKeyDown={handleKeyDown}
            style={{
              width: "100%",
              padding: "8px 10px",
              borderRadius: 4,
              border: error ? "1px solid #e53e3e" : "1px solid #4a5568",
              background: "#1a2744",
              color: "#e2e8f0",
              fontSize: 14,
              marginBottom: error ? 4 : 16,
            }}
          />
          {error && (
            <p
              data-testid="prompt-dialog-error"
              style={{
                color: "#fc8181",
                fontSize: 12,
                marginBottom: 12,
                marginTop: 0,
              }}
            >
              {error}
            </p>
          )}
          <div className="dialog-actions">
            <button
              type="button"
              onClick={onCancel}
              data-testid="prompt-dialog-cancel-btn"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="primary"
              data-testid="prompt-dialog-confirm-btn"
              disabled={!value.trim()}
            >
              OK
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
