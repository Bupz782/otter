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
      // The backend accepts only the raw text (CreateIntentRequest in
      // crates/interfaces/src/bin/metis_api.rs) and re-parses it server-side.
      // `parsed` and `delegationId` drive client-side checks and the confirm
      // summary; sending them is a backend follow-up.
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
