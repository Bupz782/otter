import { useState } from "react";
import { api, mapBackendIntent } from "@/lib/api";
import type { Intent, IntentType } from "@/types/app";

export function useCreateIntent() {
  const [data, setData] = useState<Intent | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = async (payload: {
    rawText: string;
    parsed: {
      type: IntentType;
      amount: number;
      asset: string;
      protocol: string;
      condition?: string;
      chain: string;
    };
    delegationId: string;
  }) => {
    setIsLoading(true);
    setError(null);
    try {
      const { id } = await api.intents.create(payload.rawText);
      const record = await api.intents.get(id);
      const intent = mapBackendIntent(record);
      setData(intent);
      return intent;
    } catch (err) {
      setError(err instanceof Error ? err : new Error("Failed to create intent"));
      throw err;
    } finally {
      setIsLoading(false);
    }
  };

  return { data, isLoading, error, mutate };
}
