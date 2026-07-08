import { useEffect, useState } from "react";
import { useAccount } from "wagmi";
import { api, mapBackendIntent } from "@/lib/api";
import type { Intent, IntentStatus, IntentType } from "@/types/app";

export function useIntents(filters?: { status?: IntentStatus; type?: IntentType }) {
  useAccount();
  const [data, setData] = useState<Intent[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    setIsLoading(true);
    api.intents
      .list()
      .then((res) => {
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
      .catch((err) => setError(err instanceof Error ? err : new Error(String(err))))
      .finally(() => setIsLoading(false));
  }, [filters?.status, filters?.type]);

  return { data, isLoading, error };
}
