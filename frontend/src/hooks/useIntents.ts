import { useEffect, useState } from "react";
import { useAccount } from "wagmi";
import { api, mapBackendIntent } from "@/lib/api";
import { demoIntents } from "@/lib/demo-data";
import { useAuthToken } from "@/hooks/useAuthToken";
import type { Intent, IntentStatus, IntentType } from "@/types/app";

export function useIntents(filters?: { status?: IntentStatus; type?: IntentType }) {
  useAccount();
  const { isAuthenticated } = useAuthToken();
  const isDemo = !isAuthenticated;
  const [data, setData] = useState<Intent[]>(() => (isAuthenticated ? [] : demoIntents));
  const [isLoading, setIsLoading] = useState(isAuthenticated);
  const [error, setError] = useState<Error | null>(null);
  const [reloadKey, setReloadKey] = useState(0);

  useEffect(() => {
    if (!isAuthenticated) {
      // Demo mode: serve the fixtures with the same filters applied.
      let intents = demoIntents;
      if (filters?.status) {
        intents = intents.filter((i) => i.status === filters.status);
      }
      if (filters?.type) {
        intents = intents.filter((i) => i.parsed.type === filters.type);
      }
      setData(intents);
      setError(null);
      setIsLoading(false);
      return;
    }
    let cancelled = false;
    setIsLoading(true);
    api.intents
      .list()
      .then((res) => {
        if (cancelled) return;
        let intents = res.intents.map(mapBackendIntent);
        if (filters?.status) {
          intents = intents.filter((i) => i.status === filters.status);
        }
        if (filters?.type) {
          intents = intents.filter((i) => i.parsed.type === filters.type);
        }
        setData(intents);
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
  }, [isAuthenticated, reloadKey, filters?.status, filters?.type]);

  const refetch = () => setReloadKey((key) => key + 1);

  return { data, isLoading, error, refetch, isDemo };
}
