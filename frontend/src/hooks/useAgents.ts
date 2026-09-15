import { useEffect, useState } from "react";
import { api, mapBackendAgent } from "@/lib/api";
import { demoAgents } from "@/lib/demo-data";
import type { Agent } from "@/types/app";

// The agents list endpoint is public (the Agents page is the transparency
// surface — it must work before sign-in). Always fetch it; demoAgents is only
// the offline fallback when the API cannot be reached at all.
export function useAgents() {
  const [data, setData] = useState<Agent[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const [reloadKey, setReloadKey] = useState(0);

  useEffect(() => {
    let cancelled = false;
    setIsLoading(true);
    api.agents
      .list()
      .then((res) => {
        if (cancelled) return;
        setData(res.agents.map(mapBackendAgent));
        setError(null);
      })
      .catch((err) => {
        if (cancelled) return;
        setData(demoAgents);
        setError(err instanceof Error ? err : new Error(String(err)));
      })
      .finally(() => {
        if (!cancelled) setIsLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [reloadKey]);

  const refetch = () => setReloadKey((key) => key + 1);

  return { data, isLoading, error, refetch };
}
