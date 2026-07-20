import { useEffect, useState } from "react";
import { api, mapBackendDelegation } from "@/lib/api";
import { demoDelegations } from "@/lib/demo-data";
import { useAuthToken } from "@/hooks/useAuthToken";
import type { Delegation } from "@/types/app";

export function useDelegations() {
  const { isAuthenticated } = useAuthToken();
  const isDemo = !isAuthenticated;
  const [data, setData] = useState<Delegation[]>(() => (isAuthenticated ? [] : demoDelegations));
  const [isLoading, setIsLoading] = useState(isAuthenticated);
  const [error, setError] = useState<Error | null>(null);
  const [reloadKey, setReloadKey] = useState(0);

  useEffect(() => {
    if (!isAuthenticated) {
      setData(demoDelegations);
      setError(null);
      setIsLoading(false);
      return;
    }
    let cancelled = false;
    setIsLoading(true);
    api.delegations
      .list()
      .then((res) => {
        if (cancelled) return;
        setData(res.delegations.map(mapBackendDelegation));
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
  }, [isAuthenticated, reloadKey]);

  const refetch = () => setReloadKey((key) => key + 1);

  return { data, isLoading, error, refetch, isDemo };
}
