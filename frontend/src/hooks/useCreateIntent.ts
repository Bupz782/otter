import { useState } from "react";
import { api, mapBackendIntent } from "@/lib/api";
import type { Intent, ParsedIntent } from "@/types/app";

export function useCreateIntent() {
  const [data, setData] = useState<Intent | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = async (payload: {
    rawText: string;
    parsed: ParsedIntent;
    delegationId: string;
  }) => {
    setIsLoading(true);
    setError(null);
    try {
      // The backend re-parses the raw text server-side; `parsed` drives the
      // client-side review. `delegationId` is recorded for traceability —
      // the intent and delegation pages link to each other.
      const { id } = await api.intents.create(payload.rawText, payload.delegationId);
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
