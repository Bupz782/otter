import { useEffect, useState } from "react";
import { api, mapBackendIntent } from "@/lib/api";
import type { ActivityItem } from "@/types/app";

export function useActivity() {
  const [data, setData] = useState<ActivityItem[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    setIsLoading(true);
    Promise.all([api.intents.list(), api.executions.list()])
      .then(([intentsRes, execsRes]) => {
        const intents = intentsRes.intents.map(mapBackendIntent);
        const items: ActivityItem[] = [];

        for (const intent of intents.slice(0, 10)) {
          items.push({
            id: `intent-created-${intent.id}`,
            type: "intent_created",
            title: `Created intent: ${intent.rawText}`,
            timestamp: intent.createdAt,
          });
        }

        for (const exec of execsRes.executions.slice(0, 10)) {
          items.push({
            id: exec.id,
            type: "intent_executed",
            title: `Executed intent ${exec.intent_id}`,
            timestamp: new Date(exec.created_at * 1000).toISOString(),
            txHash: exec.tx_hash,
          });
        }

        items.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());
        setData(items.slice(0, 10));
        setError(null);
      })
      .catch((err) => setError(err instanceof Error ? err : new Error(String(err))))
      .finally(() => setIsLoading(false));
  }, []);

  return { data, isLoading, error };
}
