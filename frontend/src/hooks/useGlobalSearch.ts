import { useCallback, useState } from "react";

export interface GlobalSearchResult {
  type: string;
  id: string;
  label: string;
  path: string;
}

export function useGlobalSearch() {
  const [results, setResults] = useState<GlobalSearchResult[]>([]);
  const [loading, setLoading] = useState(false);

  const search = useCallback(
    async (query: string): Promise<GlobalSearchResult[]> => {
      setLoading(true);
      try {
        console.log("TODO: search", query);
        const nextResults: GlobalSearchResult[] = [];
        setResults(nextResults);
        return nextResults;
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  return { results, loading, search };
}
