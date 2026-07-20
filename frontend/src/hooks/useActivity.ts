import { useEffect, useState } from "react";
import { api, mapBackendIntent } from "@/lib/api";
import { demoActivity } from "@/lib/demo-data";
import { useAuthToken } from "@/hooks/useAuthToken";
import type { ActivityItem } from "@/types/app";

export function useActivity() {
  const { isAuthenticated } = useAuthToken();
  const isDemo = !isAuthenticated;
  const [data, setData] = useState<ActivityItem[]>(() => (isAuthenticated ? [] : demoActivity));
  const [isLoading, setIsLoading] = useState(isAuthenticated);
  const [error, setError] = useState<Error | null>(null);
  const [reloadKey, setReloadKey] = useState(0);

  useEffect(() => {
    if (!isAuthenticated) {
      setData(demoActivity);
      setError(null);
      setIsLoading(false);
      return;
    }
    let cancelled = false;
    setIsLoading(true);
    Promise.all([api.intents.list(), api.executions.list()])
      .then(([intentsRes, execsRes]) => {
        if (cancelled) return;
        const intents = intentsRes.intents.map(mapBackendIntent);
        const textById = new Map(intents.map((intent) => [intent.id, intent.rawText]));
        const items: ActivityItem[] = [];

        for (const intent of intents.slice(0, 10)) {
          items.push({
            id: `intent-created-${intent.id}`,
            type: "intent_created",
            title: `Intent created: ${intent.rawText}`,
            timestamp: intent.createdAt,
          });
        }

        for (const exec of execsRes.executions.slice(0, 10)) {
          const rawText = textById.get(exec.intent_id);
          items.push({
            id: exec.id,
            type: "intent_executed",
            title: rawText ? `Intent executed: ${rawText}` : "Intent executed",
            timestamp: new Date(exec.created_at * 1000).toISOString(),
            txHash: exec.tx_hash,
          });
        }

        items.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());
        setData(items.slice(0, 10));
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
