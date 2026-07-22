import { useState } from "react";
import { useGlobalSearch } from "../hooks/useGlobalSearch";

export default function SearchTab() {
  const [query, setQuery] = useState("");
  const { results, loading, search } = useGlobalSearch();

  const handleChange = (value: string) => {
    setQuery(value);
    void search(value);
  };

  return (
    <section className="search-tab" data-testid="bottom-tabpanel-search">
      <input
        className="bottom-dock-search-input"
        type="search"
        value={query}
        onChange={(event) => handleChange(event.target.value)}
        placeholder="Search scenes, assets, and source files…"
        aria-label="Global search"
      />
      {loading ? (
        <p className="bottom-dock-empty">Searching…</p>
      ) : results.length > 0 ? (
        <ul className="bottom-dock-results">
          {results.map((result) => (
            <li key={`${result.type}:${result.id}`}>
              <span>{result.label}</span>
              <small>{result.path}</small>
            </li>
          ))}
        </ul>
      ) : (
        <p className="bottom-dock-empty">Search coming in v0.81</p>
      )}
    </section>
  );
}
