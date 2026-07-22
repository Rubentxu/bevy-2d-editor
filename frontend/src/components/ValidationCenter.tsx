import { useEffect, useState } from "react";
import {
  getValidationIssues,
  ValidationIssue,
} from "../services/validation-center";

interface Props {
  onClose: () => void;
}

type GroupedIssues = {
  error: ValidationIssue[];
  warning: ValidationIssue[];
  info: ValidationIssue[];
};

function groupBySeverity(issues: ValidationIssue[]): GroupedIssues {
  return {
    error: issues.filter((i) => i.severity === "error"),
    warning: issues.filter((i) => i.severity === "warning"),
    info: issues.filter((i) => i.severity === "info"),
  };
}

function SeverityIcon({
  severity,
}: {
  severity: "error" | "warning" | "info";
}) {
  if (severity === "error")
    return (
      <span className="vc-severity-icon vc-severity-error" title="Error">
        ⛔
      </span>
    );
  if (severity === "warning")
    return (
      <span className="vc-severity-icon vc-severity-warning" title="Warning">
        ⚠️
      </span>
    );
  return (
    <span className="vc-severity-icon vc-severity-info" title="Info">
      ℹ️
    </span>
  );
}

function CategoryBadge({
  category,
}: {
  category: ValidationIssue["category"];
}) {
  return (
    <span className={`vc-category-badge vc-category-${category}`}>
      {category}
    </span>
  );
}

function IssueItem({ issue }: { issue: ValidationIssue }) {
  return (
    <li
      className={`vc-issue vc-issue-${issue.severity}`}
      data-testid={`vc-issue-${issue.id}`}
    >
      <SeverityIcon severity={issue.severity} />
      <CategoryBadge category={issue.category} />
      <span className="vc-issue-code">{issue.code}</span>
      <span className="vc-issue-message">{issue.message}</span>
      {issue.affected_entity_id && (
        <span className="vc-issue-ref" title="Affected entity">
          📍 {issue.affected_entity_id}
        </span>
      )}
      {issue.affected_asset_id && (
        <span className="vc-issue-ref" title="Affected asset">
          🗃️ {issue.affected_asset_id}
        </span>
      )}
      {issue.affected_scene_id && (
        <span className="vc-issue-ref" title="Affected scene">
          📄 {issue.affected_scene_id}
        </span>
      )}
    </li>
  );
}

function IssueGroup({
  label,
  issues,
  severity,
}: {
  label: string;
  issues: ValidationIssue[];
  severity: "error" | "warning" | "info";
}) {
  if (issues.length === 0) return null;
  return (
    <section
      className={`vc-group vc-group-${severity}`}
      data-testid={`vc-group-${severity}`}
    >
      <h3 className="vc-group-label">
        <SeverityIcon severity={severity} />
        {label}
        <span className="vc-group-count">{issues.length}</span>
      </h3>
      <ul className="vc-issue-list">
        {issues.map((issue) => (
          <IssueItem key={issue.id} issue={issue} />
        ))}
      </ul>
    </section>
  );
}

export default function ValidationCenter({ onClose }: Props) {
  const [issues, setIssues] = useState<ValidationIssue[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchIssues = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await getValidationIssues();
      setIssues(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchIssues();
  }, []);

  const grouped = groupBySeverity(issues);
  const totalCount = issues.length;
  const errorCount = grouped.error.length;
  const warningCount = grouped.warning.length;

  return (
    <aside className="validation-center" data-testid="validation-center">
      <header className="vc-header">
        <h2 className="vc-title">
          Validation Center
          {totalCount > 0 && (
            <span className="vc-total-badge" data-testid="vc-total-badge">
              {totalCount}
            </span>
          )}
        </h2>
        <div className="vc-header-actions">
          <button
            className="vc-refresh-btn"
            onClick={fetchIssues}
            disabled={loading}
            title="Refresh"
            data-testid="vc-refresh-btn"
          >
            ↻
          </button>
          <button
            className="vc-close-btn"
            onClick={onClose}
            title="Close"
            data-testid="vc-close-btn"
          >
            ✕
          </button>
        </div>
      </header>

      <div className="vc-body">
        {loading && (
          <p className="vc-loading" data-testid="vc-loading">
            Loading...
          </p>
        )}
        {error && (
          <p className="vc-error" data-testid="vc-error">
            {error}
          </p>
        )}

        {!loading && !error && totalCount === 0 && (
          <div className="vc-empty" data-testid="vc-empty">
            <p>✅ No issues found — project is healthy</p>
          </div>
        )}

        {!loading && !error && totalCount > 0 && (
          <>
            <p className="vc-summary" data-testid="vc-summary">
              {errorCount > 0 && (
                <span className="vc-summary-errors">
                  {errorCount} error{errorCount !== 1 ? "s" : ""}
                </span>
              )}
              {errorCount > 0 && warningCount > 0 && ", "}
              {warningCount > 0 && (
                <span className="vc-summary-warnings">
                  {warningCount} warning{warningCount !== 1 ? "s" : ""}
                </span>
              )}
            </p>
            <IssueGroup
              label="Errors"
              issues={grouped.error}
              severity="error"
            />
            <IssueGroup
              label="Warnings"
              issues={grouped.warning}
              severity="warning"
            />
            <IssueGroup label="Info" issues={grouped.info} severity="info" />
          </>
        )}
      </div>
    </aside>
  );
}
