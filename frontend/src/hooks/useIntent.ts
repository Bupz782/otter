import { useEffect, useState } from "react";
import { api, mapBackendIntent } from "@/lib/api";
import { demoIntents } from "@/lib/demo-data";
import { useAuthToken } from "@/hooks/useAuthToken";
import type { Intent } from "@/types/app";

export function useIntent(id: string | undefined) {
  const { isAuthenticated } = useAuthToken();
  const isDemo = !isAuthenticated;
  const [data, setData] = useState<Intent | null>(() =>
    !isAuthenticated && id ? (demoIntents.find((i) => i.id === id) ?? null) : null
  );
  const [isLoading, setIsLoading] = useState(isAuthenticated && !!id);
  const [error, setError] = useState<Error | null>(null);
  const [reloadKey, setReloadKey] = useState(0);

  useEffect(() => {
    if (!isAuthenticated) {
      // Demo mode: resolve the matching fixture; unknown ids stay not-found.
      setData(id ? (demoIntents.find((i) => i.id === id) ?? null) : null);
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
    api.intents
      .get(id)
      .then((record) => {
        if (cancelled) return;
        setData(mapBackendIntent(record));
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
