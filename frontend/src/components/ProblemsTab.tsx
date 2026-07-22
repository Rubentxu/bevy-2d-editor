import { useEffect, useMemo, useState } from "react";
import {
  getValidationIssues,
  type ValidationIssue,
} from "../services/validation-center";

const SEVERITIES: ValidationIssue["severity"][] = ["error", "warning", "info"];
const SEVERITY_LABELS: Record<ValidationIssue["severity"], string> = {
  error: "Errors",
  warning: "Warnings",
  info: "Info",
};
const SEVERITY_ICONS: Record<ValidationIssue["severity"], string> = {
  error: "⛔",
  warning: "⚠",
  info: "ℹ",
};

export default function ProblemsTab() {
  const [issues, setIssues] = useState<ValidationIssue[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    void getValidationIssues()
      .then((nextIssues) => {
        if (!cancelled) setIssues(nextIssues);
      })
      .catch((error) => {
        console.warn("[ProblemsTab] validation unavailable:", error);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const grouped = useMemo(
    () =>
      SEVERITIES.map((severity) => ({
        severity,
        issues: issues.filter((issue) => issue.severity === severity),
      })),
    [issues],
  );

  return (
    <section className="problems-tab" data-testid="bottom-tabpanel-problems">
      {loading ? (
        <p className="bottom-dock-empty">Loading validation issues…</p>
      ) : issues.length === 0 ? (
        <p className="bottom-dock-empty">No problems found.</p>
      ) : (
        grouped.map(({ severity, issues: severityIssues }) =>
          severityIssues.length > 0 ? (
            <section className="problems-group" key={severity}>
              <h3>
                {SEVERITY_ICONS[severity]} {SEVERITY_LABELS[severity]}
                <span className="bottom-dock-badge">
                  {severityIssues.length}
                </span>
              </h3>
              <ul>
                {severityIssues.map((issue) => (
                  <li key={issue.id}>
                    <code>{issue.code}</code>
                    <span>{issue.message}</span>
                  </li>
                ))}
              </ul>
            </section>
          ) : null,
        )
      )}
    </section>
  );
}
