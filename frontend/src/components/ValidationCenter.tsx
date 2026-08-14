import { useEffect, useState } from "react";
import {
  getAllValidationIssues,
  type ValidationIssue,
} from "../services/validation-center";

interface Props {
  onClose: () => void;
  /** Called when the user requests navigation to the surface owning an issue. */
  onNavigate?: (issue: ValidationIssue) => void;
}

// ── Domain grouping ─────────────────────────────────────────────────────────────

type Domain = ValidationIssue["domain"];

const DOMAIN_LABELS: Record<Domain, string> = {
  scene: "Scene",
  asset: "Asset",
  logic: "Logic",
  code: "Code",
  runtime: "Runtime",
  ai: "AI",
};

const DOMAIN_ORDER: Domain[] = [
  "scene",
  "asset",
  "logic",
  "code",
  "runtime",
  "ai",
];

interface GroupedIssues {
  scene: ValidationIssue[];
  asset: ValidationIssue[];
  logic: ValidationIssue[];
  code: ValidationIssue[];
  runtime: ValidationIssue[];
  ai: ValidationIssue[];
}

function groupByDomain(issues: ValidationIssue[]): GroupedIssues {
  const g: GroupedIssues = {
    scene: [],
    asset: [],
    logic: [],
    code: [],
    runtime: [],
    ai: [],
  };
  for (const issue of issues) {
    const bucket = g[issue.domain];
    if (bucket) {
      bucket.push(issue);
    } else {
      g["runtime"].push(issue);
    }
  }
  return g;
}

// ── Filter state ───────────────────────────────────────────────────────────────

type SeverityFilter = "all" | "error" | "warning" | "info";

function useFilters() {
  const [activeDomains, setActiveDomains] = useState<Set<Domain>>(
    new Set(DOMAIN_ORDER),
  );
  const [severity, setSeverity] = useState<SeverityFilter>("all");

  const toggleDomain = (domain: Domain) => {
    setActiveDomains((prev) => {
      const next = new Set(prev);
      if (next.has(domain)) {
        // Keep at least one domain selected.
        if (next.size > 1) next.delete(domain);
      } else {
        next.add(domain);
      }
      return next;
    });
  };

  const isDomainActive = (d: Domain) => activeDomains.has(d);

  const filteredIssues = (issues: ValidationIssue[]): ValidationIssue[] => {
    return issues.filter(
      (iss) =>
        activeDomains.has(iss.domain) &&
        (severity === "all" || iss.severity === severity),
    );
  };

  return {
    activeDomains,
    severity,
    setSeverity,
    toggleDomain,
    isDomainActive,
    filteredIssues,
  };
}

// ── Severity icon ──────────────────────────────────────────────────────────────

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

// ── Issue row ──────────────────────────────────────────────────────────────────

function IssueRow({
  issue,
  isSelected,
  onSelect,
  onNavigate,
}: {
  issue: ValidationIssue;
  isSelected: boolean;
  onSelect: (issue: ValidationIssue) => void;
  onNavigate?: (issue: ValidationIssue) => void;
}) {
  return (
    <li
      className={`vc-issue vc-issue-${issue.severity}${isSelected ? " vc-issue--selected" : ""}`}
      data-testid={`vc-issue-${issue.id}`}
      onClick={() => onSelect(issue)}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onSelect(issue);
        }
      }}
    >
      <SeverityIcon severity={issue.severity} />
      <span className={`vc-domain-badge vc-domain-${issue.domain}`}>
        {DOMAIN_LABELS[issue.domain]}
      </span>
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

// ── Left sidebar ────────────────────────────────────────────────────────────────

function Sidebar({
  grouped,
  filters,
  totalCount,
}: {
  grouped: GroupedIssues;
  filters: ReturnType<typeof useFilters>;
  totalCount: number;
}) {
  const domainCounts = DOMAIN_ORDER.map((d) => ({
    domain: d,
    label: DOMAIN_LABELS[d],
    count: filters.filteredIssues(grouped[d]).length,
    isActive: filters.isDomainActive(d),
  }));

  const activeErrorCount = DOMAIN_ORDER.reduce(
    (sum, d) => sum + grouped[d].filter((i) => i.severity === "error").length,
    0,
  );

  return (
    <aside className="vc-sidebar" data-testid="vc-sidebar" aria-label="Filters">
      <div className="vc-sidebar__section">
        <p className="vc-sidebar__section-label">Severity</p>
        {(["all", "error", "warning", "info"] as SeverityFilter[]).map((s) => (
          <button
            key={s}
            className={`vc-sidebar__filter-btn${filters.severity === s ? " vc-sidebar__filter-btn--active" : ""}`}
            onClick={() => filters.setSeverity(s)}
            data-testid={`vc-severity-filter-${s}`}
          >
            {s === "all" ? "All" : s.charAt(0).toUpperCase() + s.slice(1)}
            {s === "error" && activeErrorCount > 0 && (
              <span className="vc-sidebar__count vc-sidebar__count--error">
                {activeErrorCount}
              </span>
            )}
          </button>
        ))}
      </div>

      <div className="vc-sidebar__section">
        <p className="vc-sidebar__section-label">Domain</p>
        {domainCounts.map(({ domain, label, count, isActive }) => (
          <button
            key={domain}
            className={`vc-sidebar__filter-btn${!isActive ? " vc-sidebar__filter-btn--muted" : ""}`}
            onClick={() => filters.toggleDomain(domain)}
            data-testid={`vc-domain-filter-${domain}`}
            aria-pressed={isActive}
          >
            <span className={`vc-domain-dot vc-domain-dot--${domain}`} />
            {label}
            <span className="vc-sidebar__count">{count}</span>
          </button>
        ))}
      </div>

      <div className="vc-sidebar__footer">
        <span className="vc-sidebar__total">
          {totalCount} issue{totalCount !== 1 ? "s" : ""}
        </span>
      </div>
    </aside>
  );
}

// ── Center: issue list ─────────────────────────────────────────────────────────

function IssueList({
  grouped,
  filters,
  selectedIssue,
  onSelect,
}: {
  grouped: GroupedIssues;
  filters: ReturnType<typeof useFilters>;
  selectedIssue: ValidationIssue | null;
  onSelect: (issue: ValidationIssue) => void;
}) {
  const nonEmptyDomains = DOMAIN_ORDER.filter(
    (d) => filters.filteredIssues(grouped[d]).length > 0,
  );

  if (nonEmptyDomains.length === 0) {
    return (
      <div className="vc-list vc-list--empty" data-testid="vc-list-empty">
        <p>No issues match the current filters.</p>
      </div>
    );
  }

  return (
    <ul className="vc-list" data-testid="vc-list" role="list">
      {nonEmptyDomains.map((domain) => {
        const domainIssues = filters.filteredIssues(grouped[domain]);
        if (domainIssues.length === 0) return null;
        return (
          <li key={domain} className="vc-list__domain-group">
            <h3
              className="vc-list__domain-label"
              data-testid={`vc-domain-header-${domain}`}
            >
              <span className={`vc-domain-dot vc-domain-dot--${domain}`} />
              {DOMAIN_LABELS[domain]}
              <span className="vc-list__domain-count">
                {domainIssues.length}
              </span>
            </h3>
            <ul className="vc-list__issues">
              {domainIssues.map((issue) => (
                <IssueRow
                  key={issue.id}
                  issue={issue}
                  isSelected={selectedIssue?.id === issue.id}
                  onSelect={onSelect}
                />
              ))}
            </ul>
          </li>
        );
      })}
    </ul>
  );
}

// ── Right: issue detail ────────────────────────────────────────────────────────

function IssueDetail({
  issue,
  onNavigate,
  onClose,
}: {
  issue: ValidationIssue;
  onNavigate?: (issue: ValidationIssue) => void;
  onClose: () => void;
}) {
  return (
    <aside
      className="vc-detail"
      data-testid="vc-detail"
      aria-label="Issue detail"
    >
      <header className="vc-detail__header">
        <SeverityIcon severity={issue.severity} />
        <span className={`vc-domain-badge vc-domain-${issue.domain}`}>
          {DOMAIN_LABELS[issue.domain]}
        </span>
        <span className="vc-detail__code">{issue.code}</span>
        <button
          className="vc-detail__close"
          onClick={onClose}
          data-testid="vc-detail-close"
          title="Close detail"
        >
          ✕
        </button>
      </header>

      <p className="vc-detail__message">{issue.message}</p>

      <dl className="vc-detail__refs">
        {issue.affected_entity_id && (
          <>
            <dt>Entity</dt>
            <dd className="vc-detail__ref-value">{issue.affected_entity_id}</dd>
          </>
        )}
        {issue.affected_asset_id && (
          <>
            <dt>Asset</dt>
            <dd className="vc-detail__ref-value">{issue.affected_asset_id}</dd>
          </>
        )}
        {issue.affected_scene_id && (
          <>
            <dt>Scene</dt>
            <dd className="vc-detail__ref-value">{issue.affected_scene_id}</dd>
          </>
        )}
      </dl>

      {onNavigate && (
        <div className="vc-detail__actions">
          <button
            className="vc-btn vc-btn--primary"
            onClick={() => onNavigate(issue)}
            data-testid="vc-detail-navigate"
          >
            Go to source
          </button>
        </div>
      )}
    </aside>
  );
}

// ── Main component ─────────────────────────────────────────────────────────────

export default function ValidationCenter({ onClose, onNavigate }: Props) {
  const [issues, setIssues] = useState<ValidationIssue[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedIssue, setSelectedIssue] = useState<ValidationIssue | null>(
    null,
  );
  const filters = useFilters();

  const fetchIssues = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await getAllValidationIssues();
      setIssues(result);
      // Auto-select first error if nothing selected.
      if (!selectedIssue && result.length > 0) {
        const firstError =
          result.find((i) => i.severity === "error") ?? result[0];
        setSelectedIssue(firstError);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void fetchIssues();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const grouped = groupByDomain(issues);
  const totalCount = issues.length;
  const errorCount = issues.filter((i) => i.severity === "error").length;
  const warningCount = issues.filter((i) => i.severity === "warning").length;

  const handleSelect = (issue: ValidationIssue) => setSelectedIssue(issue);
  const handleDetailClose = () => setSelectedIssue(null);

  return (
    <aside
      className="validation-center validation-center--inbox"
      data-testid="validation-center"
    >
      {/* Header */}
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
          {(errorCount > 0 || warningCount > 0) && (
            <span className="vc-header-summary" data-testid="vc-header-summary">
              {errorCount > 0 && (
                <span className="vc-header-summary__errors">
                  {errorCount} error{errorCount !== 1 ? "s" : ""}
                </span>
              )}
              {errorCount > 0 && warningCount > 0 && ", "}
              {warningCount > 0 && (
                <span className="vc-header-summary__warnings">
                  {warningCount} warning{warningCount !== 1 ? "s" : ""}
                </span>
              )}
            </span>
          )}
          <button
            className="vc-refresh-btn"
            onClick={() => void fetchIssues()}
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

      {/* 3-column inbox body */}
      <div className="vc-body vc-body--inbox">
        {/* Left: filters */}
        <Sidebar grouped={grouped} filters={filters} totalCount={totalCount} />

        {/* Center: grouped issue list */}
        <section className="vc-center" aria-label="Issue list">
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
            <IssueList
              grouped={grouped}
              filters={filters}
              selectedIssue={selectedIssue}
              onSelect={handleSelect}
            />
          )}
        </section>

        {/* Right: issue detail — only shown when an issue is selected */}
        {selectedIssue && (
          <IssueDetail
            issue={selectedIssue}
            onNavigate={onNavigate}
            onClose={handleDetailClose}
          />
        )}
      </div>
    </aside>
  );
}
