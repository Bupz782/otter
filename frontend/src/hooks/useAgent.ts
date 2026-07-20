import { useEffect, useState } from "react";
import { api, mapBackendAgent } from "@/lib/api";
import { demoAgents } from "@/lib/demo-data";
import { useAuthToken } from "@/hooks/useAuthToken";
import type { Agent } from "@/types/app";

export function useAgent(id: string | undefined) {
  const { isAuthenticated } = useAuthToken();
  const isDemo = !isAuthenticated;
  const [data, setData] = useState<Agent | null>(() =>
    !isAuthenticated && id ? (demoAgents.find((a) => a.id === id) ?? null) : null
  );
  const [isLoading, setIsLoading] = useState(isAuthenticated && !!id);
  const [error, setError] = useState<Error | null>(null);
  const [reloadKey, setReloadKey] = useState(0);

  useEffect(() => {
    if (!isAuthenticated) {
      // Demo mode: resolve the matching fixture; unknown ids stay not-found.
      setData(id ? (demoAgents.find((a) => a.id === id) ?? null) : null);
      setError(null);
      setIsLoading(false);
      return;
    }
    if (!id) {
      setData(null);
      setError(null);
      setIsLoading(false);
      return;
    }
    let cancelled = false;
    setIsLoading(true);
    api.agents
      .get(id)
      .then((res) => {
        if (cancelled) return;
        setData(mapBackendAgent(res));
        setError(null);
      })
      .catch((err) => {
        if (cancelled) return;
        setError(err instanceof Error ? err : new Error(String(err)));
      })
      .finally(() => {
        if (!cancelled) setIsLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [id, isAuthenticated, reloadKey]);

  const refetch = () => setReloadKey((key) => key + 1);

  return { data, isLoading, error, refetch, isDemo };
}
